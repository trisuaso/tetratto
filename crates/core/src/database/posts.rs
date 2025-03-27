use super::*;
use crate::cache::Cache;
use crate::model::communities::PostContext;
use crate::model::{
    Error, Result,
    auth::User,
    communities::{CommunityWriteAccess, Post},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row, query_rows};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Post`] from an SQL row.
    pub(crate) fn get_post_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Post {
        Post {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            content: get!(x->2(String)),
            owner: get!(x->3(i64)) as usize,
            community: get!(x->4(i64)) as usize,
            context: serde_json::from_str(&get!(x->5(String))).unwrap(),
            replying_to: if let Some(id) = get!(x->6(Option<i64>)) {
                Some(id as usize)
            } else {
                None
            },
            // likes
            likes: get!(x->7(i64)) as isize,
            dislikes: get!(x->8(i64)) as isize,
            // other counts
            comment_count: get!(x->9(i64)) as usize,
        }
    }

    auto_method!(get_post_by_id()@get_post_from_row -> "SELECT * FROM posts WHERE id = $1" --name="post" --returns=Post --cache-key-tmpl="atto.post:{}");

    /// Get all posts which are comments on the given post by ID.
    ///
    /// # Arguments
    /// * `id` - the ID of the post the requested posts are commenting on
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_post_comments(&self, id: usize, batch: usize, page: usize) -> Result<Post> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM posts WHERE replying_to = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Ok(Self::get_post_from_row(x)) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all posts from the given user (from most recent).
    ///
    /// # Arguments
    /// * `id` - the ID of the user the requested posts belong to
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_posts_by_user(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Post>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM posts WHERE owner = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new journal entry in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`JournalEntry`] object to insert
    pub async fn create_post(&self, data: Post) -> Result<()> {
        // check values
        if data.content.len() < 2 {
            return Err(Error::DataTooShort("content".to_string()));
        } else if data.content.len() > 4096 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        // check permission in page
        let page = match self.get_page_by_id(data.community).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        match page.write_access {
            CommunityWriteAccess::Owner => {
                if data.owner != page.owner {
                    return Err(Error::NotAllowed);
                }
            }
            CommunityWriteAccess::Joined => {
                if let Err(_) = self
                    .get_membership_by_owner_community(data.owner, page.id)
                    .await
                {
                    return Err(Error::NotAllowed);
                }
            }
            _ => (),
        };

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO posts VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &Some(data.id.to_string()),
                &Some(data.created.to_string()),
                &Some(data.content),
                &Some(data.owner.to_string()),
                &Some(data.community.to_string()),
                &Some(serde_json::to_string(&data.context).unwrap()),
                &if let Some(id) = data.replying_to {
                    Some(id.to_string())
                } else {
                    None
                },
                &Some(0.to_string()),
                &Some(0.to_string()),
                &Some(0.to_string())
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // incr comment count
        if let Some(id) = data.replying_to {
            self.incr_post_comments(id).await.unwrap();
        }

        // return
        Ok(())
    }

    auto_method!(delete_post()@get_post_by_id:MANAGE_JOURNAL_ENTRIES -> "DELETE FROM posts WHERE id = $1" --cache-key-tmpl="atto.post:{}");
    auto_method!(update_post_content(String)@get_post_by_id:MANAGE_JOURNAL_ENTRIES -> "UPDATE posts SET content = $1 WHERE id = $2" --cache-key-tmpl="atto.post:{}");
    auto_method!(update_post_context(PostContext)@get_post_by_id:MANAGE_JOURNAL_ENTRIES -> "UPDATE posts SET context = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.post:{}");

    auto_method!(incr_post_likes() -> "UPDATE posts SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(incr_post_dislikes() -> "UPDATE posts SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_likes() -> "UPDATE posts SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
    auto_method!(decr_post_dislikes() -> "UPDATE posts SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);

    auto_method!(incr_post_comments() -> "UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_comments() -> "UPDATE posts SET comment_count = comment_count - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
}
