use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::IpBan, auth::User, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`IpBan`] from an SQL row.
    pub(crate) fn get_ipban_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> IpBan {
        IpBan {
            ip: get!(x->0(String)),
            created: get!(x->1(i64)) as usize,
            reason: get!(x->2(String)),
            moderator: get!(x->3(i64)) as usize,
        }
    }

    auto_method!(get_ipban_by_ip(&str)@get_ipban_from_row -> "SELECT * FROM ipbans WHERE ip = $1" --name="ip ban" --returns=IpBan --cache-key-tmpl="atto.ipban:{}");

    /// Create a new user block in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`IpBan`] object to insert
    pub async fn create_ipban(&self, data: IpBan) -> Result<()> {
        let user = self.get_user_by_id(data.moderator).await?;

        // ONLY moderators can create ip bans
        if !user.permissions.check(FinePermission::MANAGE_BANS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO ipbans VALUES ($1, $2, $3, $4)",
            &[
                &data.ip.as_str(),
                &data.created.to_string().as_str(),
                &data.reason.as_str(),
                &data.moderator.to_string().as_str()
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_ipban(&self, id: usize, user: User) -> Result<()> {
        // ONLY moderators can manage ip bans
        if !user.permissions.check(FinePermission::MANAGE_BANS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM ipbans WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.ipban:{}", id)).await;

        // return
        Ok(())
    }
}
