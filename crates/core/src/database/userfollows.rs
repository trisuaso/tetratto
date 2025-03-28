use super::*;
use crate::cache::Cache;
use crate::model::{Error, Result, auth::User, auth::UserFollow, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`UserFollow`] from an SQL row.
    pub(crate) fn get_userfollow_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> UserFollow {
        UserFollow {
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            initiator: get!(x->2(isize)) as usize,
            receiver: get!(x->3(isize)) as usize,
        }
    }

    auto_method!(get_userfollow_by_id()@get_userfollow_from_row -> "SELECT * FROM userfollows WHERE id = $1" --name="user follow" --returns=UserFollow --cache-key-tmpl="atto.userfollow:{}");

    /// Get a user follow by `initiator` and `receiver` (in that order).
    pub async fn get_userfollow_by_initiator_receiver(
        &self,
        initiator: usize,
        receiver: usize,
    ) -> Result<UserFollow> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM userfollows WHERE initiator = $1 AND receiver = $2",
            &[&(initiator as i64), &(receiver as i64)],
            |x| { Ok(Self::get_userfollow_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get a user follow by `receiver` and `initiator` (in that order).
    pub async fn get_userfollow_by_receiver_initiator(
        &self,
        receiver: usize,
        initiator: usize,
    ) -> Result<UserFollow> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM userfollows WHERE receiver = $1 AND initiator = $2",
            &[&(receiver as i64), &(initiator as i64)],
            |x| { Ok(Self::get_userfollow_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new user follow in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`UserFollow`] object to insert
    pub async fn create_userfollow(&self, data: UserFollow) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO userfollows VALUES ($1, $2, $3, $4)",
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

        // incr counts
        self.incr_user_following_count(data.initiator)
            .await
            .unwrap();

        self.incr_user_follower_count(data.receiver).await.unwrap();

        // return
        Ok(())
    }

    pub async fn delete_userfollow(&self, id: usize, user: User) -> Result<()> {
        let follow = self.get_userfollow_by_id(id).await?;

        if (user.id != follow.initiator) && (user.id != follow.receiver) {
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
            "DELETE FROM userfollows WHERE id = $1",
            &[&id.to_string()]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.userfollow:{}", id)).await;

        // decr counts
        self.incr_user_following_count(follow.initiator)
            .await
            .unwrap();

        self.incr_user_follower_count(follow.receiver)
            .await
            .unwrap();

        // return
        Ok(())
    }
}
