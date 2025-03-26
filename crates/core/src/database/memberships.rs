use super::*;
use crate::cache::Cache;
use crate::model::{
    Error, Result, auth::User, journal::JournalMembership, journal_permissions::JournalPermission,
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`JournalEntry`] from an SQL row.
    pub(crate) fn get_membership_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> JournalMembership {
        JournalMembership {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            journal: get!(x->3(i64)) as usize,
            role: JournalPermission::from_bits(get!(x->4(u32))).unwrap(),
        }
    }

    auto_method!(get_membership_by_id()@get_membership_from_row -> "SELECT * FROM memberships WHERE id = $1" --name="journal membership" --returns=JournalMembership --cache-key-tmpl="atto.membership:{}");

    /// Get a journal membership by `owner` and `journal`.
    pub async fn get_membership_by_owner_journal(
        &self,
        owner: usize,
        journal: usize,
    ) -> Result<JournalMembership> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM memberships WHERE owner = $1 AND journal = $2",
            &[&(owner as i64), &(journal as i64)],
            |x| { Ok(Self::get_membership_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("journal membership".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new journal membership in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`JournalMembership`] object to insert
    pub async fn create_membership(&self, data: JournalMembership) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO memberships VALUES ($1, $2, $3, $4, $5",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.owner.to_string().as_str(),
                &data.journal.to_string().as_str(),
                &(data.role.bits()).to_string().as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        Ok(())
    }

    /// Delete a membership given its `id`
    pub async fn delete_membership(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_membership_by_id(id).await?;

        if user.id != y.owner {
            // pull other user's membership status
            if let Ok(z) = self.get_membership_by_id(user.id).await {
                // somebody with MANAGE_ROLES _and_ a higher role number can remove us
                if (!z.role.check(JournalPermission::MANAGE_ROLES) | (z.role < y.role))
                    && !z.role.check(JournalPermission::ADMINISTRATOR)
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
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.membership:{}", id)).await;

        Ok(())
    }

    /// Update a membership's role given its `id`
    pub async fn update_membership_role(
        &self,
        id: usize,
        new_role: JournalPermission,
    ) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE memberships SET role = $1 WHERE id = $2",
            &[&(new_role.bits()).to_string(), &id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.membership:{}", id)).await;

        Ok(())
    }
}
