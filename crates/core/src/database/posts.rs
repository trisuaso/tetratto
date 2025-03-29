use super::*;
use crate::cache::Cache;
use crate::model::{
    Error, Result,
    auth::User,
    communities::{Community, CommunityWriteAccess, Post, PostContext},
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
            id: get!(x->0(isize)) as usize,
            created: get!(x->1(isize)) as usize,
            content: get!(x->2(String)),
            owner: get!(x->3(isize)) as usize,
            community: get!(x->4(isize)) as usize,
            context: serde_json::from_str(&get!(x->5(String))).unwrap(),
            replying_to: if let Some(id) = get!(x->6(Option<i64>)) {
                Some(id as usize)
            } else {
                None
            },
            // likes
            likes: get!(x->7(isize)) as isize,
            dislikes: get!(x->8(isize)) as isize,
            // other counts
            comment_count: get!(x->9(isize)) as usize,
        }
    }

    auto_method!(get_post_by_id()@get_post_from_row -> "SELECT * FROM posts WHERE id = $1" --name="post" --returns=Post --cache-key-tmpl="atto.post:{}");

    /// Get all posts which are comments on the given post by ID.
    ///
    /// # Arguments
    /// * `id` - the ID of the post the requested posts are commenting on
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_post_comments(
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
            "SELECT * FROM posts WHERE replying_to = $1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Complete a vector of just posts with their owner as well.
    pub async fn fill_posts(&self, posts: Vec<Post>) -> Result<Vec<(Post, User)>> {
        let mut out: Vec<(Post, User)> = Vec::new();

        for post in posts {
            let owner = post.owner.clone();
            out.push((post, self.get_user_by_id(owner).await?));
        }

        Ok(out)
    }

    /// Complete a vector of just posts with their owner and community as well.
    pub async fn fill_posts_with_community(
        &self,
        posts: Vec<Post>,
    ) -> Result<Vec<(Post, User, Community)>> {
        let mut out: Vec<(Post, User, Community)> = Vec::new();

        for post in posts {
            let owner = post.owner.clone();
            let community = post.community.clone();
            out.push((
                post,
                self.get_user_by_id(owner).await?,
                self.get_community_by_id(community).await?,
            ));
        }

        Ok(out)
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

    /// Get all posts from the given community (from most recent).
    ///
    /// # Arguments
    /// * `id` - the ID of the community the requested posts belong to
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_posts_by_community(
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
            "SELECT * FROM posts WHERE community = $1 AND replying_to IS NULL ORDER BY created DESC LIMIT $2 OFFSET $3",
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
    pub async fn create_post(&self, data: Post) -> Result<usize> {
        // check values
        if data.content.len() < 2 {
            return Err(Error::DataTooShort("content".to_string()));
        } else if data.content.len() > 4096 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        // check permission in community
        let community = match self.get_community_by_id(data.community).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        match community.write_access {
            CommunityWriteAccess::Owner => {
                if data.owner != community.owner {
                    return Err(Error::NotAllowed);
                }
            }
            CommunityWriteAccess::Joined => {
                if let Err(_) = self
                    .get_membership_by_owner_community(data.owner, community.id)
                    .await
                {
                    return Err(Error::NotAllowed);
                }
            }
            _ => (),
        };

        // check if we're blocked
        if let Some(replying_to) = data.replying_to {
            if let Ok(_) = self
                .get_userblock_by_initiator_receiver(replying_to, data.owner)
                .await
            {
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
            "INSERT INTO posts VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
        Ok(data.id)
    }

    pub async fn delete_post(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_post_by_id(id).await?;

        if user.id != y.owner {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(&conn, "DELETE FROM posts WHERE id = $1", &[&id.to_string()]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.post:{}", id)).await;

        // decr parent comment count
        if let Some(replying_to) = y.replying_to {
            self.decr_post_comments(replying_to).await.unwrap();
        }

        // return
        Ok(())
    }

    auto_method!(update_post_content(String)@get_post_by_id:MANAGE_POSTS -> "UPDATE posts SET content = $1 WHERE id = $2" --cache-key-tmpl="atto.post:{}");
    auto_method!(update_post_context(PostContext)@get_post_by_id:MANAGE_POSTS -> "UPDATE posts SET context = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.post:{}");

    auto_method!(incr_post_likes() -> "UPDATE posts SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(incr_post_dislikes() -> "UPDATE posts SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_likes() -> "UPDATE posts SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
    auto_method!(decr_post_dislikes() -> "UPDATE posts SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);

    auto_method!(incr_post_comments() -> "UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_comments() -> "UPDATE posts SET comment_count = comment_count - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
}
