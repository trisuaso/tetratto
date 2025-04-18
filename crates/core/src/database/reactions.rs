use super::*;
use crate::cache::Cache;
use crate::model::{
    Error, Result,
    auth::{Notification, User},
    permissions::FinePermission,
    reactions::{AssetType, Reaction},
};
use crate::{auto_method, execute, get, query_row, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Reaction`] from an SQL row.
    pub(crate) fn get_reaction_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Reaction {
        Reaction {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            asset: get!(x->3(i64)) as usize,
            asset_type: serde_json::from_str(&get!(x->4(String))).unwrap(),
            is_like: get!(x->5(i32)) as i8 == 1,
        }
    }

    auto_method!(get_reaction_by_id()@get_reaction_from_row -> "SELECT * FROM reactions WHERE id = $1" --name="reaction" --returns=Reaction --cache-key-tmpl="atto.reaction:{}");

    /// Get a reaction by `owner` and `asset`.
    pub async fn get_reaction_by_owner_asset(
        &self,
        owner: usize,
        asset: usize,
    ) -> Result<Reaction> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM reactions WHERE owner = $1 AND asset = $2",
            &[&(owner as i64), &(asset as i64)],
            |x| { Ok(Self::get_reaction_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("reaction".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new reaction in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Reaction`] object to insert
    pub async fn create_reaction(&self, data: Reaction, user: &User) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO reactions VALUES ($1, $2, $3, $4, $5, $6)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.owner as i64),
                &(data.asset as i64),
                &serde_json::to_string(&data.asset_type).unwrap().as_str(),
                &{ if data.is_like { 1 } else { 0 } }
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // incr corresponding
        match data.asset_type {
            AssetType::Community => {
                if let Err(e) = {
                    if data.is_like {
                        self.incr_community_likes(data.asset).await
                    } else {
                        self.incr_community_dislikes(data.asset).await
                    }
                } {
                    return Err(e);
                } else if data.is_like {
                    let community = self.get_community_by_id_no_void(data.asset).await.unwrap();

                    if community.owner != user.id {
                        self
                            .create_notification(Notification::new(
                                "Your community has received a like!".to_string(),
                                format!(
                                    "[@{}](/api/v1/auth/user/find/{}) has liked your [community](/api/v1/communities/find/{})!",
                                    user.username, user.id, community.id
                                ),
                                community.owner,
                            ))
                            .await?
                    }
                }
            }
            AssetType::Post => {
                if let Err(e) = {
                    if data.is_like {
                        self.incr_post_likes(data.asset).await
                    } else {
                        self.incr_post_dislikes(data.asset).await
                    }
                } {
                    return Err(e);
                } else if data.is_like {
                    let post = self.get_post_by_id(data.asset).await.unwrap();

                    if post.owner != user.id {
                        self.create_notification(Notification::new(
                            "Your post has received a like!".to_string(),
                            format!(
                                "[@{}](/api/v1/auth/user/find/{}) has liked your [post](/post/{})!",
                                user.username, user.id, data.asset
                            ),
                            post.owner,
                        ))
                        .await?
                    }
                }
            }
            AssetType::Question => {
                if let Err(e) = {
                    if data.is_like {
                        self.incr_question_likes(data.asset).await
                    } else {
                        self.incr_question_dislikes(data.asset).await
                    }
                } {
                    return Err(e);
                } else if data.is_like {
                    let question = self.get_question_by_id(data.asset).await.unwrap();

                    if question.owner != user.id {
                        self
                            .create_notification(Notification::new(
                                "Your question has received a like!".to_string(),
                                format!(
                                    "[@{}](/api/v1/auth/user/find/{}) has liked your [question](/question/{})!",
                                    user.username, user.id, data.asset
                                ),
                                question.owner,
                            ))
                            .await?
                    }
                }
            }
            AssetType::User => {
                return Err(Error::NotAllowed);
            }
        };

        // return
        Ok(())
    }

    pub async fn delete_reaction(&self, id: usize, user: &User) -> Result<()> {
        let reaction = self.get_reaction_by_id(id).await?;

        if user.id != reaction.owner && !user.permissions.check(FinePermission::MANAGE_REACTIONS) {
            return Err(Error::NotAllowed);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM reactions WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.reaction:{}", id)).await;

        // decr corresponding
        match reaction.asset_type {
            AssetType::Community => {
                if reaction.is_like {
                    self.decr_community_likes(reaction.asset).await
                } else {
                    self.decr_community_dislikes(reaction.asset).await
                }
            }?,
            AssetType::Post => {
                if reaction.is_like {
                    self.decr_post_likes(reaction.asset).await
                } else {
                    self.decr_post_dislikes(reaction.asset).await
                }
            }?,
            AssetType::Question => {
                if reaction.is_like {
                    self.decr_question_likes(reaction.asset).await
                } else {
                    self.decr_question_dislikes(reaction.asset).await
                }
            }?,
            AssetType::User => {
                return Err(Error::NotAllowed);
            }
        };

        // return
        Ok(())
    }
}
