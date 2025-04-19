use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::User, auth::IpBlock, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`UserBlock`] from an SQL row.
    pub(crate) fn get_ipblock_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> IpBlock {
        IpBlock {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            initiator: get!(x->2(i64)) as usize,
            receiver: get!(x->3(String)),
        }
    }

    auto_method!(get_ipblock_by_id()@get_ipblock_from_row -> "SELECT * FROM ipblocks WHERE id = $1" --name="ip block" --returns=IpBlock --cache-key-tmpl="atto.ipblock:{}");

    /// Get a user block by `initiator` and `receiver` (in that order).
    pub async fn get_ipblock_by_initiator_receiver(
        &self,
        initiator: usize,
        receiver: &str,
    ) -> Result<IpBlock> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM ipblocks WHERE initiator = $1 AND receiver = $2",
            params![&(initiator as i64), &receiver],
            |x| { Ok(Self::get_ipblock_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user block".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get a user block by `receiver` and `initiator` (in that order).
    pub async fn get_ipblock_by_receiver_initiator(
        &self,
        receiver: &str,
        initiator: usize,
    ) -> Result<IpBlock> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM ipblocks WHERE receiver = $1 AND initiator = $2",
            params![&receiver, &(initiator as i64)],
            |x| { Ok(Self::get_ipblock_from_row(x)) }
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
    pub async fn create_ipblock(&self, data: IpBlock) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO ipblocks VALUES ($1, $2, $3, $4)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.initiator as i64),
                &data.receiver
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_ipblock(&self, id: usize, user: User) -> Result<()> {
        let block = self.get_ipblock_by_id(id).await?;

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

        let res = execute!(&conn, "DELETE FROM ipblocks WHERE id = $1", &[&(id as i64)]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.ipblock:{}", id)).await;

        // return
        Ok(())
    }
}
