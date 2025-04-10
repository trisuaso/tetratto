use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::Notification, auth::User, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Notification`] from an SQL row.
    pub(crate) fn get_notification_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Notification {
        Notification {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            title: get!(x->2(String)),
            content: get!(x->3(String)),
            owner: get!(x->4(i64)) as usize,
            read: get!(x->5(i32)) as i8 == 1,
        }
    }

    auto_method!(get_notification_by_id()@get_notification_from_row -> "SELECT * FROM notifications WHERE id = $1" --name="notification" --returns=Notification --cache-key-tmpl="atto.notification:{}");

    /// Get all notifications by `owner`.
    pub async fn get_notifications_by_owner(&self, owner: usize) -> Result<Vec<Notification>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM notifications WHERE owner = $1 ORDER BY created DESC",
            &[&(owner as i64)],
            |x| { Self::get_notification_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("notification".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new notification in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Reaction`] object to insert
    pub async fn create_notification(&self, data: Notification) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO notifications VALUES ($1, $2, $3, $4, $5, $6)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &data.title,
                &data.content,
                &(data.owner as i64),
                &{ if data.read { 1 } else { 0 } }
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // incr notification count
        self.incr_user_notifications(data.owner).await.unwrap();

        // return
        Ok(())
    }

    pub async fn delete_notification(&self, id: usize, user: &User) -> Result<()> {
        let notification = self.get_notification_by_id(id).await?;

        if user.id != notification.owner && !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM notifications WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.notification:{}", id)).await;

        // decr notification count
        if !notification.read {
            self.decr_user_notifications(notification.owner)
                .await
                .unwrap();
        }

        // return
        Ok(())
    }

    pub async fn delete_all_notifications(&self, user: &User) -> Result<()> {
        let notifications = self.get_notifications_by_owner(user.id).await?;

        for notification in notifications {
            if user.id != notification.owner && !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
                return Err(Error::NotAllowed);
            }

            self.delete_notification(notification.id, user).await?
        }

        Ok(())
    }

    pub async fn update_notification_read(
        &self,
        id: usize,
        new_read: bool,
        user: &User,
    ) -> Result<()> {
        let y = self.get_notification_by_id(id).await?;

        if y.owner != user.id && !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
            return Err(Error::NotAllowed);
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE notifications SET read = $1 WHERE id = $2",
            params![&{ if new_read { 1 } else { 0 } }, &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.notification:{}", id)).await;

        if (y.read) && (!new_read) {
            self.incr_user_notifications(user.id).await?;
        } else if (!y.read) && (new_read) {
            self.decr_user_notifications(user.id).await?;
        }

        Ok(())
    }
}
