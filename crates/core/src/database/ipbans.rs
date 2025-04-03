use super::*;
use crate::cache::Cache;
use crate::model::moderation::AuditLogEntry;
use crate::model::{Error, Result, auth::IpBan, auth::User, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows};

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

    /// Get all IP bans (paginated).
    ///
    /// # Arguments
    /// * `batch` - the limit of items in each page
    /// * `page` - the page number
    pub async fn get_ipbans(&self, batch: usize, page: usize) -> Result<Vec<IpBan>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM ipbans ORDER BY created DESC LIMIT $1 OFFSET $2",
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_ipban_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("ip ban".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new IP ban in the database.
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

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!("invoked `create_ipban` with x value `{}`", data.ip),
        ))
        .await?;

        // return
        Ok(())
    }

    pub async fn delete_ipban(&self, ip: &str, user: User) -> Result<()> {
        // ONLY moderators can manage ip bans
        if !user.permissions.check(FinePermission::MANAGE_BANS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(&conn, "DELETE FROM ipbans WHERE ip = $1", &[&ip]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.ipban:{}", ip)).await;

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!("invoked `delete_ipban` with x value `{ip}`"),
        ))
        .await?;

        // return
        Ok(())
    }
}
