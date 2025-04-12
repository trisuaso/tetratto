use super::*;
use crate::cache::Cache;
use crate::model::auth::{Notification, UserWarning};
use crate::model::moderation::AuditLogEntry;
use crate::model::{Error, Result, auth::User, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`UserWarning`] from an SQL row.
    pub(crate) fn get_user_warning_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> UserWarning {
        UserWarning {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            receiver: get!(x->2(i64)) as usize,
            moderator: get!(x->3(i64)) as usize,
            content: get!(x->4(String)),
        }
    }

    auto_method!(get_user_warning_by_ip(&str)@get_user_warning_from_row -> "SELECT * FROM user_warning WHERE ip = $1" --name="user warning" --returns=UserWarning --cache-key-tmpl="atto.user_warning:{}");

    /// Get all user warnings by user (paginated).
    ///
    /// # Arguments
    /// * `user` - the ID of the user to fetch warnings for
    /// * `batch` - the limit of items in each page
    /// * `page` - the page number
    pub async fn get_user_warnings_by_user(
        &self,
        user: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<UserWarning>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM user_warnings WHERE receiver = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(user as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_user_warning_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user warning".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new user warning in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`UserWarning`] object to insert
    pub async fn create_user_warning(&self, data: UserWarning) -> Result<()> {
        let user = self.get_user_by_id(data.moderator).await?;

        // ONLY moderators can create warnings
        if !user.permissions.check(FinePermission::MANAGE_WARNINGS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO user_warnings VALUES ($1, $2, $3, $4, $5)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.receiver as i64),
                &(data.moderator as i64),
                &data.content
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!(
                "invoked `create_user_warning` with x value `{}`",
                data.receiver
            ),
        ))
        .await?;

        // send notification
        self.create_notification(Notification::new(
            "You have received a new account warning.".to_string(),
            data.content,
            data.receiver,
        ))
        .await?;

        // return
        Ok(())
    }

    pub async fn delete_user_warning(&self, id: usize, user: User) -> Result<()> {
        // ONLY moderators can manage warnings
        if !user.permissions.check(FinePermission::MANAGE_WARNINGS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM user_warnings WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.user_warning:{}", id)).await;

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!("invoked `delete_user_warning` with x value `{id}`"),
        ))
        .await?;

        // return
        Ok(())
    }
}
