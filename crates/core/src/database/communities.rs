use super::*;
use crate::cache::Cache;
use crate::model::communities::{CommunityContext, CommunityJoinAccess, CommunityMembership};
use crate::model::communities_permissions::CommunityPermission;
use crate::model::{
    Error, Result,
    auth::User,
    communities::Community,
    communities::{CommunityReadAccess, CommunityWriteAccess},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};
use pathbufd::PathBufD;
use std::fs::{exists, remove_file};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Community`] from an SQL row.
    pub(crate) fn get_community_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Community {
        Community {
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            title: get!(x->2(String)),
            context: serde_json::from_str(&get!(x->3(String))).unwrap(),
            owner: get!(x->4(isize)) as usize,
            read_access: serde_json::from_str(&get!(x->5(String))).unwrap(),
            write_access: serde_json::from_str(&get!(x->6(String))).unwrap(),
            join_access: serde_json::from_str(&get!(x->7(String))).unwrap(),
            // likes
            likes: get!(x->8(isize)) as isize,
            dislikes: get!(x->9(isize)) as isize,
            // counts
            member_count: get!(x->10(isize)) as usize,
        }
    }

    pub async fn get_community_by_id(&self, id: usize) -> Result<Community> {
        if id == 0 {
            return Ok(Community::void());
        }

        if let Some(cached) = self.2.get(format!("atto.community:{}", id)).await {
            return Ok(serde_json::from_str(&cached).unwrap());
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM communities WHERE id = $1",
            &[&(id as isize)],
            |x| { Ok(Self::get_community_from_row(x)) }
        );

        if res.is_err() {
            return Ok(Community::void());
            // return Err(Error::GeneralNotFound("community".to_string()));
        }

        let x = res.unwrap();
        self.2
            .set(
                format!("atto.community:{}", id),
                serde_json::to_string(&x).unwrap(),
            )
            .await;

        Ok(x)
    }

    pub async fn get_community_by_title(&self, id: &str) -> Result<Community> {
        if id == "void" {
            return Ok(Community::void());
        }

        if let Some(cached) = self.2.get(format!("atto.community:{}", id)).await {
            return Ok(serde_json::from_str(&cached).unwrap());
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM communities WHERE title = $1",
            &[id],
            |x| { Ok(Self::get_community_from_row(x)) }
        );

        if res.is_err() {
            return Ok(Community::void());
            // return Err(Error::GeneralNotFound("community".to_string()));
        }

        let x = res.unwrap();
        self.2
            .set(
                format!("atto.community:{}", id),
                serde_json::to_string(&x).unwrap(),
            )
            .await;

        Ok(x)
    }

    auto_method!(get_community_by_id_no_void()@get_community_from_row -> "SELECT * FROM communities WHERE id = $1" --name="community" --returns=Community --cache-key-tmpl="atto.community:{}");
    auto_method!(get_community_by_title_no_void(&str)@get_community_from_row -> "SELECT * FROM communities WHERE title = $1" --name="community" --returns=Community --cache-key-tmpl="atto.community:{}");

    /// Create a new community in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Community`] to insert
    pub async fn create_community(&self, data: Community) -> Result<String> {
        // check values
        if data.title.len() < 2 {
            return Err(Error::DataTooShort("title".to_string()));
        } else if data.title.len() > 32 {
            return Err(Error::DataTooLong("title".to_string()));
        }

        if !data.title.is_ascii() | data.title.contains(" ") {
            return Err(Error::MiscError(
                "Title contains characters which aren't allowed".to_string(),
            ));
        }

        if self.0.banned_usernames.contains(&data.title) {
            return Err(Error::MiscError("This title cannot be used".to_string()));
        }

        // check number of communities
        let memberships = self.get_memberships_by_owner(data.owner).await?;
        let mut admin_count = 0; // you can not make anymore communities if you are already admin of at least 5

        for membership in memberships {
            if membership.role.check(CommunityPermission::ADMINISTRATOR) {
                admin_count += 1;
            }
        }

        if admin_count >= 5 {
            return Err(Error::MiscError(
                "You are already owner/co-owner of too many communities to create another"
                    .to_string(),
            ));
        }

        // make sure community doesn't already exist with title
        if self
            .get_community_by_title_no_void(&data.title.to_lowercase())
            .await
            .is_ok()
        {
            return Err(Error::MiscError("Title already in use".to_string()));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO communities VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.title.to_lowercase().as_str(),
                &serde_json::to_string(&data.context).unwrap().as_str(),
                &data.owner.to_string().as_str(),
                &serde_json::to_string(&data.read_access).unwrap().as_str(),
                &serde_json::to_string(&data.write_access).unwrap().as_str(),
                &serde_json::to_string(&data.join_access).unwrap().as_str(),
                &0.to_string().as_str(),
                &0.to_string().as_str(),
                &0.to_string().as_str()
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // add journal page owner as admin
        self.create_membership(CommunityMembership::new(
            data.owner,
            data.id,
            CommunityPermission::ADMINISTRATOR,
        ))
        .await
        .unwrap();

        // return
        Ok(data.title)
    }

    pub async fn cache_clear_community(&self, community: &Community) {
        self.2
            .remove(format!("atto.community:{}", community.id))
            .await;
        self.2
            .remove(format!("atto.community:{}", community.title))
            .await;
    }

    pub async fn delete_community(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_community_by_id(id).await?;

        if user.id != y.owner {
            if !user.permissions.check(FinePermission::MANAGE_COMMUNITIES) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(crate::model::moderation::AuditLogEntry::new(
                    user.id,
                    format!("invoked `delete_community` with x value `{id}`"),
                ))
                .await?
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM communities WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_community(&y).await;

        // remove memberships
        let res = execute!(
            &conn,
            "DELETE FROM memberships WHERE community = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // remove images
        let avatar = PathBufD::current().extend(&[
            self.0.dirs.media.as_str(),
            "community_avatars",
            &format!("{}.avif", &y.id),
        ]);

        let banner = PathBufD::current().extend(&[
            self.0.dirs.media.as_str(),
            "community_banners",
            &format!("{}.avif", &y.id),
        ]);

        if exists(&avatar).unwrap() {
            remove_file(avatar).unwrap();
        }

        if exists(&banner).unwrap() {
            remove_file(banner).unwrap();
        }

        // ...
        Ok(())
    }

    auto_method!(update_community_title(String)@get_community_by_id_no_void:MANAGE_COMMUNITIES -> "UPDATE communities SET title = $1 WHERE id = $2" --cache-key-tmpl=cache_clear_community);
    auto_method!(update_community_context(CommunityContext)@get_community_by_id_no_void:MANAGE_COMMUNITIES -> "UPDATE communities SET context = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_community);
    auto_method!(update_community_read_access(CommunityReadAccess)@get_community_by_id_no_void:MANAGE_COMMUNITIES -> "UPDATE communities SET read_access = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_community);
    auto_method!(update_community_write_access(CommunityWriteAccess)@get_community_by_id_no_void:MANAGE_COMMUNITIES -> "UPDATE communities SET write_access = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_community);
    auto_method!(update_community_join_access(CommunityJoinAccess)@get_community_by_id_no_void:MANAGE_COMMUNITIES -> "UPDATE communities SET join_access = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_community);

    auto_method!(incr_community_likes()@get_community_by_id_no_void -> "UPDATE communities SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --incr);
    auto_method!(incr_community_dislikes()@get_community_by_id_no_void -> "UPDATE communities SET dislikes = dislikes + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --incr);
    auto_method!(decr_community_likes()@get_community_by_id_no_void -> "UPDATE communities SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --decr);
    auto_method!(decr_community_dislikes()@get_community_by_id_no_void -> "UPDATE communities SET dislikes = dislikes - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --decr);

    auto_method!(incr_community_member_count()@get_community_by_id_no_void -> "UPDATE communities SET member_count = member_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --incr);
    auto_method!(decr_community_member_count()@get_community_by_id_no_void -> "UPDATE communities SET member_count = member_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_community --decr);
}
