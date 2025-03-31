use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::User, auth::UserBlock, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`UserBlock`] from an SQL row.
    pub(crate) fn get_userblock_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> UserBlock {
        UserBlock {
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            initiator: get!(x->2(isize)) as usize,
            receiver: get!(x->3(isize)) as usize,
        }
    }

    auto_method!(get_userblock_by_id()@get_userblock_from_row -> "SELECT * FROM userblocks WHERE id = $1" --name="user block" --returns=UserBlock --cache-key-tmpl="atto.userblock:{}");

    /// Get a user block by `initiator` and `receiver` (in that order).
    pub async fn get_userblock_by_initiator_receiver(
        &self,
        initiator: usize,
        receiver: usize,
    ) -> Result<UserBlock> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM userblocks WHERE initiator = $1 AND receiver = $2",
            &[&(initiator as isize), &(receiver as isize)],
            |x| { Ok(Self::get_userblock_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user block".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get a user block by `receiver` and `initiator` (in that order).
    pub async fn get_userblock_by_receiver_initiator(
        &self,
        receiver: usize,
        initiator: usize,
    ) -> Result<UserBlock> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM userblocks WHERE receiver = $1 AND initiator = $2",
            &[&(receiver as isize), &(initiator as isize)],
            |x| { Ok(Self::get_userblock_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user block".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new user block in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`UserBlock`] object to insert
    pub async fn create_userblock(&self, data: UserBlock) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO userblocks VALUES ($1, $2, $3, $4)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.initiator.to_string().as_str(),
                &data.receiver.to_string().as_str()
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_userblock(&self, id: usize, user: User) -> Result<()> {
        let block = self.get_userblock_by_id(id).await?;

        if user.id != block.initiator {
            // only the initiator (or moderators) can delete user blocks!
            if !user.permissions.check(FinePermission::MANAGE_FOLLOWS) {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM userblocks WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.userblock:{}", id)).await;

        // return
        Ok(())
    }
}
