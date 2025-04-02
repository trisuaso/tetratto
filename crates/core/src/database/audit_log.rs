use super::*;
use crate::cache::Cache;
use crate::model::{
    Error, Result, auth::User, moderation::AuditLogEntry, permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get an [`AuditLogEntry`] from an SQL row.
    pub(crate) fn get_auditlog_entry_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            moderator: get!(x->2(isize)) as usize,
            content: get!(x->3(String)),
        }
    }

    auto_method!(get_auditlog_entry_by_id(usize)@get_auditlog_entry_from_row -> "SELECT * FROM auditlog WHERE id = $1" --name="audit log entry" --returns=AuditLogEntry --cache-key-tmpl="atto.auditlog:{}");

    /// Create a new audit log entry in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`AuditLogEntry`] object to insert
    pub async fn create_auditlog_entry(&self, data: AuditLogEntry) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO auditlog VALUES ($1, $2, $3, $4)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.moderator.to_string().as_str(),
                &data.content.as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_auditlog_entry(&self, id: usize, user: User) -> Result<()> {
        if !user.permissions.check(FinePermission::MANAGE_AUDITLOG) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM auditlog WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.auditlog:{}", id)).await;

        // return
        Ok(())
    }
}
