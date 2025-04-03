use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::User, moderation::AuditLogEntry, permissions::FinePermission};
use crate::{auto_method, execute, get, params, query_row, query_rows};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get an [`AuditLogEntry`] from an SQL row.
    pub(crate) fn get_audit_log_entry_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            moderator: get!(x->2(i64)) as usize,
            content: get!(x->3(String)),
        }
    }

    auto_method!(get_audit_log_entry_by_id(usize as i64)@get_audit_log_entry_from_row -> "SELECT * FROM audit_log WHERE id = $1" --name="audit log entry" --returns=AuditLogEntry --cache-key-tmpl="atto.audit_log:{}");

    /// Get all audit log entries (paginated).
    ///
    /// # Arguments
    /// * `batch` - the limit of items in each page
    /// * `page` - the page number
    pub async fn get_audit_log_entries(
        &self,
        batch: usize,
        page: usize,
    ) -> Result<Vec<AuditLogEntry>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM audit_log ORDER BY created DESC LIMIT $1 OFFSET $2",
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_audit_log_entry_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("audit log entry".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new audit log entry in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`AuditLogEntry`] object to insert
    pub async fn create_audit_log_entry(&self, data: AuditLogEntry) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO audit_log VALUES ($1, $2, $3, $4)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.moderator as i64),
                &data.content.as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_audit_log_entry(&self, id: usize, user: User) -> Result<()> {
        if !user.permissions.check(FinePermission::MANAGE_AUDITLOG) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM audit_log WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.audit_log:{}", id)).await;

        // return
        Ok(())
    }
}
