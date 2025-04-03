use super::*;
use crate::cache::Cache;
use crate::model::auth::Notification;
use crate::model::communities::Community;
use crate::model::{
    Error, Result,
    auth::User,
    communities::{CommunityJoinAccess, CommunityMembership},
    communities_permissions::CommunityPermission,
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`JournalEntry`] from an SQL row.
    pub(crate) fn get_membership_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> CommunityMembership {
        CommunityMembership {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            community: get!(x->3(i64)) as usize,
            role: CommunityPermission::from_bits(get!(x->4(i32)) as u32).unwrap(),
        }
    }

    auto_method!(get_membership_by_id()@get_membership_from_row -> "SELECT * FROM memberships WHERE id = $1" --name="community membership" --returns=CommunityMembership --cache-key-tmpl="atto.membership:{}");

    /// Replace a list of community memberships with the proper community.
    pub async fn fill_communities(&self, list: Vec<CommunityMembership>) -> Result<Vec<Community>> {
        let mut communities: Vec<Community> = Vec::new();
        for membership in &list {
            communities.push(self.get_community_by_id(membership.community).await?);
        }
        Ok(communities)
    }

    /// Get a community membership by `owner` and `community`.
    pub async fn get_membership_by_owner_community(
        &self,
        owner: usize,
        community: usize,
    ) -> Result<CommunityMembership> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM memberships WHERE owner = $1 AND community = $2",
            &[&(owner as i64), &(community as i64)],
            |x| { Ok(Self::get_membership_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("community membership".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all community memberships by `owner`.
    pub async fn get_memberships_by_owner(&self, owner: usize) -> Result<Vec<CommunityMembership>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            // 33 = banned, 65 = pending membership
            "SELECT * FROM memberships WHERE owner = $1 AND NOT role = 33 AND NOT role = 65 ORDER BY created DESC",
            &[&(owner as i64)],
            |x| { Self::get_membership_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("community membership".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new community membership in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`CommunityMembership`] object to insert
    #[async_recursion::async_recursion]
    pub async fn create_membership(&self, data: CommunityMembership) -> Result<String> {
        // make sure membership doesn't already exist
        if self
            .get_membership_by_owner_community(data.owner, data.community)
            .await
            .is_ok()
        {
            return Err(Error::MiscError("Already joined community".to_string()));
        }

        // check permission
        let community = self.get_community_by_id(data.community).await?;

        match community.join_access {
            CommunityJoinAccess::Nobody => return Err(Error::NotAllowed),
            CommunityJoinAccess::Request => {
                if !data.role.check(CommunityPermission::REQUESTED) {
                    let mut data = data.clone();
                    data.role = CommunityPermission::DEFAULT | CommunityPermission::REQUESTED;

                    // send notification to the owner
                    self.create_notification(Notification::new(
                        "You've received a community join request!".to_string(),
                        format!(
                            "[Somebody](/api/v1/auth/profile/find/{}) is asking to join your [community](/community/{}).\n\n[Click here to review their request](/community/{}/manage?uid={}#/members).",
                            data.owner, data.community, data.community, data.owner
                        ),
                        community.owner,
                    ))
                    .await?;

                    // ...
                    return self.create_membership(data).await;
                }
            }
            _ => (),
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO memberships VALUES ($1, $2, $3, $4, $5)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.owner as i64),
                &(data.community as i64),
                &(data.role.bits() as i32),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        if !data.role.check(CommunityPermission::REQUESTED) {
            // users who are just a requesting to join do not count towards the member count
            self.incr_community_member_count(data.community)
                .await
                .unwrap();
        }

        Ok(if data.role.check(CommunityPermission::REQUESTED) {
            "Join request sent".to_string()
        } else {
            "Community joined".to_string()
        })
    }

    /// Delete a membership given its `id`
    pub async fn delete_membership(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_membership_by_id(id).await?;

        if user.id != y.owner {
            // pull other user's membership status
            if let Ok(z) = self
                .get_membership_by_owner_community(user.id, y.community)
                .await
            {
                // somebody with MANAGE_ROLES _and_ a higher role number can remove us
                if (!z.role.check(CommunityPermission::MANAGE_ROLES) | (z.role < y.role))
                    && !z.role.check(CommunityPermission::ADMINISTRATOR)
                {
                    return Err(Error::NotAllowed);
                }
            } else if !user.permissions.check(FinePermission::MANAGE_MEMBERSHIPS) {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM memberships WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.membership:{}", id)).await;

        self.decr_community_member_count(y.community).await.unwrap();

        Ok(())
    }

    /// Update a membership's role given its `id`
    pub async fn update_membership_role(
        &self,
        id: usize,
        new_role: CommunityPermission,
    ) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE memberships SET role = $1 WHERE id = $2",
            params![&(new_role.bits() as i64), &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.membership:{}", id)).await;

        Ok(())
    }
}
