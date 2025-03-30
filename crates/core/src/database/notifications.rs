use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::Notification, auth::User, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows};

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
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            title: get!(x->2(String)),
            content: get!(x->3(String)),
            owner: get!(x->4(isize)) as usize,
            read: if get!(x->5(i8)) == 1 { true } else { false },
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
            "SELECT * FROM notifications WHERE owner = $1",
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
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.title.to_string().as_str(),
                &data.content.to_string().as_str(),
                &data.owner.to_string().as_str(),
                &(if data.read { 1 } else { 0 }).to_string().as_str()
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

        if user.id != notification.owner {
            if !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM notification WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.notification:{}", id)).await;

        // decr notification count
        self.decr_user_notifications(notification.owner)
            .await
            .unwrap();

        // return
        Ok(())
    }

    pub async fn delete_all_notifications(&self, user: &User) -> Result<()> {
        let notifications = self.get_notifications_by_owner(user.id).await?;

        for notification in notifications {
            if user.id != notification.owner {
                if !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
                    return Err(Error::NotAllowed);
                }
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

        if y.owner != user.id {
            if !user.permissions.check(FinePermission::MANAGE_NOTIFICATIONS) {
                return Err(Error::NotAllowed);
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE notifications SET read = $1 WHERE id = $2",
            &[&(if new_read { 1 } else { 0 }).to_string(), &id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.notification:{}", id)).await;

        if (y.read == true) && (new_read == false) {
            self.incr_user_notifications(user.id).await?;
        } else if (y.read == false) && (new_read == true) {
            self.decr_user_notifications(user.id).await?;
        }

        Ok(())
    }
}
