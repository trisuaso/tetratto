use super::*;
use crate::cache::Cache;
use crate::model::auth::FollowResult;
use crate::model::requests::{ActionRequest, ActionType};
use crate::model::{Error, Result, auth::User, auth::UserFollow, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row, query_rows, params};

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
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            initiator: get!(x->2(i64)) as usize,
            receiver: get!(x->3(i64)) as usize,
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

    /// Get users the given user is following.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `batch` - the limit of userfollows in each page
    /// * `page` - the page number
    pub async fn get_userfollows_by_initiator(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<UserFollow>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM userfollows WHERE initiator = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_userfollow_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get users the given user is following.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    pub async fn get_userfollows_by_initiator_all(&self, id: usize) -> Result<Vec<UserFollow>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM userfollows WHERE initiator = $1",
            &[&(id as i64)],
            |x| { Self::get_userfollow_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get users following the given user.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `batch` - the limit of userfollows in each page
    /// * `page` - the page number
    pub async fn get_userfollows_by_receiver(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<UserFollow>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM userfollows WHERE receiver = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_userfollow_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get users following the given user.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    pub async fn get_userfollows_by_receiver_all(&self, id: usize) -> Result<Vec<UserFollow>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM userfollows WHERE receiver = $1",
            &[&(id as i64)],
            |x| { Self::get_userfollow_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("user follow".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Complete a vector of just userfollows with their receiver as well.
    pub async fn fill_userfollows_with_receiver(
        &self,
        userfollows: Vec<UserFollow>,
    ) -> Result<Vec<(UserFollow, User)>> {
        let mut out: Vec<(UserFollow, User)> = Vec::new();

        for userfollow in userfollows {
            let receiver = userfollow.receiver;
            out.push((userfollow, self.get_user_by_id(receiver).await?));
        }

        Ok(out)
    }

    /// Complete a vector of just userfollows with their initiator as well.
    pub async fn fill_userfollows_with_initiator(
        &self,
        userfollows: Vec<UserFollow>,
    ) -> Result<Vec<(UserFollow, User)>> {
        let mut out: Vec<(UserFollow, User)> = Vec::new();

        for userfollow in userfollows {
            let initiator = userfollow.initiator;
            out.push((userfollow, self.get_user_by_id(initiator).await?));
        }

        Ok(out)
    }

    /// Create a new user follow in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`UserFollow`] object to insert
    /// * `force` - if we should skip the request stage
    pub async fn create_userfollow(&self, data: UserFollow, force: bool) -> Result<FollowResult> {
        if !force {
            let other_user = self.get_user_by_id(data.receiver).await?;

            if other_user.settings.private_profile {
                // send follow request instead
                self.create_request(ActionRequest::with_id(
                    data.initiator,
                    data.receiver,
                    ActionType::Follow,
                    data.receiver,
                ))
                .await?;

                return Ok(FollowResult::Requested);
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO userfollows VALUES ($1, $2, $3, $4)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.initiator as i64),
                &(data.receiver as i64)
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
        Ok(FollowResult::Followed)
    }

    pub async fn delete_userfollow(&self, id: usize, user: &User) -> Result<()> {
        let follow = self.get_userfollow_by_id(id).await?;

        if (user.id != follow.initiator)
            && (user.id != follow.receiver)
            && !user.permissions.check(FinePermission::MANAGE_FOLLOWS)
        {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM userfollows WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.userfollow:{}", id)).await;

        // decr counts
        self.decr_user_following_count(follow.initiator)
            .await
            .unwrap();

        self.decr_user_follower_count(follow.receiver)
            .await
            .unwrap();

        // return
        Ok(())
    }
}
