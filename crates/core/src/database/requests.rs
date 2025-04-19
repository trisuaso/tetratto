use super::*;
use crate::cache::Cache;
use crate::model::requests::ActionType;
use crate::model::{Error, Result, requests::ActionRequest, auth::User, permissions::FinePermission};
use crate::{execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get an [`ActionRequest`] from an SQL row.
    pub(crate) fn get_request_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> ActionRequest {
        ActionRequest {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            action_type: serde_json::from_str(&get!(x->3(String))).unwrap(),
            linked_asset: get!(x->4(i64)) as usize,
        }
    }

    pub async fn get_request_by_id_linked_asset(
        &self,
        id: usize,
        linked_asset: usize,
    ) -> Result<ActionRequest> {
        if let Some(cached) = self
            .2
            .get(format!("atto.request:{}:{}", id, linked_asset))
            .await
        {
            return Ok(serde_json::from_str(&cached).unwrap());
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM requests WHERE id = $1 AND linked_asset = $2",
            &[&(id as i64), &(linked_asset as i64)],
            |x| { Ok(Self::get_request_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("request".to_string()));
        }

        let x = res.unwrap();
        self.2
            .set(
                format!("atto.request:{}:{}", id, linked_asset),
                serde_json::to_string(&x).unwrap(),
            )
            .await;

        Ok(x)
    }

    /// Get all action requests by `owner`.
    pub async fn get_requests_by_owner(&self, owner: usize) -> Result<Vec<ActionRequest>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM requests WHERE owner = $1 ORDER BY created DESC",
            &[&(owner as i64)],
            |x| { Self::get_request_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("request".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new request in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`ActionRequest`] object to insert
    pub async fn create_request(&self, data: ActionRequest) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO requests VALUES ($1, $2, $3, $4, $5)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.owner as i64),
                &serde_json::to_string(&data.action_type).unwrap().as_str(),
                &(data.linked_asset as i64),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // incr request count
        self.incr_user_request_count(data.owner).await.unwrap();

        // return
        Ok(())
    }

    pub async fn delete_request(
        &self,
        id: usize,
        linked_asset: usize,
        user: &User,
        force: bool,
    ) -> Result<()> {
        let y = self
            .get_request_by_id_linked_asset(id, linked_asset)
            .await?;

        if !force && user.id != y.owner && !user.permissions.check(FinePermission::MANAGE_REQUESTS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM requests WHERE id = $1",
            &[&(y.id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.request:{}", y.id)).await;

        self.2
            .remove(format!("atto.request:{}:{}", id, linked_asset))
            .await;

        // decr request count
        let owner = self.get_user_by_id(y.owner).await?;
        if owner.request_count > 0 {
            self.decr_user_request_count(y.owner).await.unwrap();
        }

        // return
        Ok(())
    }

    pub async fn delete_all_requests(&self, user: &User) -> Result<()> {
        let y = self.get_requests_by_owner(user.id).await?;

        for x in y {
            if user.id != x.owner && !user.permissions.check(FinePermission::MANAGE_REQUESTS) {
                return Err(Error::NotAllowed);
            }

            self.delete_request(x.id, x.linked_asset, user, false)
                .await?;

            // delete question
            if x.action_type == ActionType::Answer {
                self.delete_question(x.linked_asset, user).await?;
            }
        }

        self.update_user_request_count(user.id, 0).await?;

        Ok(())
    }
}
