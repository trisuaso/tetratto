use std::collections::HashMap;

use super::*;
use crate::cache::Cache;
use crate::model::auth::Notification;
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
            &[
                &(id as isize),
                &(batch as isize),
                &((page * batch) as isize)
            ],
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
            &[
                &(id as isize),
                &(batch as isize),
                &((page * batch) as isize)
            ],
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
            &[
                &(id as isize),
                &(batch as isize),
                &((page * batch) as isize)
            ],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get posts from all communities, sorted by likes.
    ///
    /// # Arguments
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_popular_posts(&self, batch: usize, page: usize) -> Result<Vec<Post>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM posts ORDER BY likes DESC, created ASC LIMIT $2 OFFSET $3",
            &[&(batch as isize), &((page * batch) as isize)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get posts from all communities the given user is in.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_posts_from_user_communities(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Post>> {
        let memberships = self.get_memberships_by_owner(id).await?;
        let mut memberships = memberships.iter();
        let first = match memberships.next() {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut query_string: String = String::new();

        for membership in memberships {
            query_string.push_str(&format!(" OR community = {}", membership.community));
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            &format!(
                "SELECT * FROM posts WHERE community = {} {query_string} ORDER BY created DESC LIMIT $1 OFFSET $2",
                first.community
            ),
            &[&(batch as isize), &((page * batch) as isize)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Check if the given `uid` can post in the given `community`.
    pub async fn check_can_post(&self, community: &Community, uid: usize) -> bool {
        match community.write_access {
            CommunityWriteAccess::Owner => {
                if uid != community.owner {
                    false
                } else {
                    true
                }
            }
            CommunityWriteAccess::Joined => {
                match self
                    .get_membership_by_owner_community(uid, community.id)
                    .await
                {
                    Ok(m) => {
                        if !m.role.check_member() {
                            false
                        } else {
                            true
                        }
                    }
                    Err(_) => false,
                }
            }
            _ => true,
        }
    }

    /// Create a new journal entry in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`JournalEntry`] object to insert
    pub async fn create_post(&self, mut data: Post) -> Result<usize> {
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

        if !self.check_can_post(&community, data.owner).await {
            return Err(Error::NotAllowed);
        }

        // check if we're blocked
        if let Some(replying_to) = data.replying_to {
            if let Ok(_) = self
                .get_userblock_by_initiator_receiver(replying_to, data.owner)
                .await
            {
                return Err(Error::NotAllowed);
            }
        }

        // send mention notifications
        let mut already_notified: HashMap<String, User> = HashMap::new();
        for username in User::parse_mentions(&data.content) {
            let user = {
                if let Some(ua) = already_notified.get(&username) {
                    ua.to_owned()
                } else {
                    let user = self.get_user_by_username(&username).await?;
                    self.create_notification(Notification::new(
                        "You've been mentioned in a post!".to_string(),
                        format!(
                            "[Somebody](/api/v1/auth/profile/find/{}) mentioned you in their [post](/post/{}).",
                            data.owner, data.id
                        ),
                        user.id,
                    ))
                    .await?;
                    already_notified.insert(username.to_owned(), user.clone());
                    user
                }
            };

            data.content = data.content.replace(
                &format!("@{username}"),
                &format!("[@{username}](/api/v1/auth/profile/find/{})", user.id),
            );
        }

        // incr comment count
        if let Some(id) = data.replying_to {
            self.incr_post_comments(id).await.unwrap();

            // send notification
            let rt = self.get_post_by_id(id).await?;

            if data.owner != rt.owner {
                let owner = self.get_user_by_id(rt.owner).await?;
                self.create_notification(Notification::new(
                    "Your post has received a new comment!".to_string(),
                    format!(
                        "[@{}](/api/v1/auth/profile/find/{}) has commented on your [post](/post/{}).",
                        owner.username, owner.id, rt.id
                    ),
                    rt.owner,
                ))
                .await?;

                if rt.context.comments_enabled == false {
                    return Err(Error::NotAllowed);
                }
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let replying_to_id = data.replying_to.unwrap_or(0).to_string();

        let res = execute!(
            &conn,
            "INSERT INTO posts VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                &Some(data.id.to_string().as_str()),
                &Some(data.created.to_string().as_str()),
                &Some(&data.content),
                &Some(data.owner.to_string().as_str()),
                &Some(data.community.to_string().as_str()),
                &Some(&serde_json::to_string(&data.context).unwrap()),
                &if replying_to_id != "0" {
                    Some(replying_to_id.as_str())
                } else {
                    None
                },
                &Some(0.to_string().as_str()),
                &Some(0.to_string().as_str()),
                &Some(0.to_string().as_str())
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(data.id)
    }

    pub async fn delete_post(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_post_by_id(id).await?;

        if user.id != y.owner {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(crate::model::moderation::AuditLogEntry::new(
                    user.id,
                    format!("invoked `delete_post` with x value `{id}`"),
                ))
                .await?
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
    auto_method!(incr_post_dislikes() -> "UPDATE posts SET dislikes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_likes() -> "UPDATE posts SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
    auto_method!(decr_post_dislikes() -> "UPDATE posts SET dislikes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);

    auto_method!(incr_post_comments() -> "UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_comments() -> "UPDATE posts SET comment_count = comment_count - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
}
