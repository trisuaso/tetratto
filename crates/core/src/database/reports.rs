use super::*;
use crate::cache::Cache;
use crate::model::moderation::AuditLogEntry;
use crate::model::{Error, Result, auth::User, moderation::Report, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Report`] from an SQL row.
    pub(crate) fn get_report_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Report {
        Report {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            content: get!(x->3(String)),
            asset: get!(x->4(i64)) as usize,
            asset_type: serde_json::from_str(&get!(x->5(String))).unwrap(),
        }
    }

    auto_method!(get_report_by_id(usize)@get_report_from_row -> "SELECT * FROM reports WHERE id = $1" --name="report" --returns=Report --cache-key-tmpl="atto.reports:{}");

    /// Get all reports (paginated).
    ///
    /// # Arguments
    /// * `batch` - the limit of items in each page
    /// * `page` - the page number
    pub async fn get_reports(&self, batch: usize, page: usize) -> Result<Vec<Report>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM reports ORDER BY created DESC LIMIT $1 OFFSET $2",
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_report_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("report".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new report in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Report`] object to insert
    pub async fn create_report(&self, data: Report) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO reports VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.owner.to_string().as_str(),
                &data.content.as_str(),
                &data.asset.to_string().as_str(),
                &serde_json::to_string(&data.asset_type).unwrap().as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_report(&self, id: usize, user: User) -> Result<()> {
        if !user.permissions.check(FinePermission::MANAGE_REPORTS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM reports WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.report:{}", id)).await;

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!("invoked `delete_report` with x value `{id}`"),
        ))
        .await?;

        // return
        Ok(())
    }
}
