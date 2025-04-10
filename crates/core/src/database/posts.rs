use std::collections::HashMap;

use super::*;
use crate::cache::Cache;
use crate::model::auth::Notification;
use crate::model::communities_permissions::CommunityPermission;
use crate::model::moderation::AuditLogEntry;
use crate::model::{
    Error, Result,
    auth::User,
    communities::{Community, CommunityWriteAccess, Post, PostContext},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

use tetratto_shared::unix_epoch_timestamp;
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
            replying_to: get!(x->6(Option<i64>)).map(|id| id as usize),
            // likes
            likes: get!(x->7(i32)) as isize,
            dislikes: get!(x->8(i32)) as isize,
            // other counts
            comment_count: get!(x->9(i32)) as usize,
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

    /// Get the post the given post is reposting (if some).
    pub async fn get_post_reposting(&self, post: &Post) -> Option<(User, Post)> {
        if let Some(ref repost) = post.context.repost {
            if let Some(reposting) = repost.reposting {
                let mut x = match self.get_post_by_id(reposting).await {
                    Ok(p) => p,
                    Err(_) => return None,
                };

                x.mark_as_repost();
                Some((
                    match self.get_user_by_id(x.owner).await {
                        Ok(ua) => ua,
                        Err(_) => return None,
                    },
                    x,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Complete a vector of just posts with their owner as well.
    pub async fn fill_posts(
        &self,
        posts: Vec<Post>,
    ) -> Result<Vec<(Post, User, Option<(User, Post)>)>> {
        let mut out: Vec<(Post, User, Option<(User, Post)>)> = Vec::new();

        let mut users: HashMap<usize, User> = HashMap::new();
        for post in posts {
            let owner = post.owner;

            if let Some(user) = users.get(&owner) {
                out.push((
                    post.clone(),
                    user.clone(),
                    self.get_post_reposting(&post).await,
                ));
            } else {
                let user = self.get_user_by_id(owner).await?;
                users.insert(owner, user.clone());
                out.push((post.clone(), user, self.get_post_reposting(&post).await));
            }
        }

        Ok(out)
    }

    /// Complete a vector of just posts with their owner and community as well.
    pub async fn fill_posts_with_community(
        &self,
        posts: Vec<Post>,
        user_id: usize,
    ) -> Result<Vec<(Post, User, Community, Option<(User, Post)>)>> {
        let mut out: Vec<(Post, User, Community, Option<(User, Post)>)> = Vec::new();

        let mut seen_before: HashMap<(usize, usize), (User, Community)> = HashMap::new();
        let mut seen_user_follow_statuses: HashMap<(usize, usize), bool> = HashMap::new();

        for post in posts {
            let owner = post.owner;
            let community = post.community;

            if let Some((user, community)) = seen_before.get(&(owner, community)) {
                out.push((
                    post.clone(),
                    user.clone(),
                    community.to_owned(),
                    self.get_post_reposting(&post).await,
                ));
            } else {
                let user = self.get_user_by_id(owner).await?;

                // check relationship
                if user.settings.private_profile {
                    if user_id == 0 {
                        continue;
                    }

                    if let Some(is_following) = seen_user_follow_statuses.get(&(user.id, user_id)) {
                        if !is_following {
                            // post owner is not following us
                            continue;
                        }
                    } else {
                        if self
                            .get_userfollow_by_initiator_receiver(user.id, user_id)
                            .await
                            .is_err()
                        {
                            // post owner is not following us
                            seen_user_follow_statuses.insert((user.id, user_id), false);
                            continue;
                        }

                        seen_user_follow_statuses.insert((user.id, user_id), true);
                    }
                }

                // ...
                let community = self.get_community_by_id(community).await?;
                seen_before.insert((owner, community.id), (user.clone(), community.clone()));
                out.push((
                    post.clone(),
                    user,
                    community,
                    self.get_post_reposting(&post).await,
                ));
            }
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
            "SELECT * FROM posts WHERE owner = $1 AND replying_to = 0 AND NOT context LIKE '%\"is_profile_pinned\":true%' AND NOT context LIKE '%\"is_nsfw\":true%' ORDER BY created DESC LIMIT $2 OFFSET $3",
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
            "SELECT * FROM posts WHERE community = $1 AND replying_to = 0 AND NOT context LIKE '%\"is_pinned\":true%' ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[&(id as i64), &(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all pinned posts from the given community (from most recent).
    ///
    /// # Arguments
    /// * `id` - the ID of the community the requested posts belong to
    pub async fn get_pinned_posts_by_community(&self, id: usize) -> Result<Vec<Post>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM posts WHERE community = $1 AND context LIKE '%\"is_pinned\":true%' ORDER BY created DESC",
            &[&(id as i64),],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all pinned posts from the given user (from most recent).
    ///
    /// # Arguments
    /// * `id` - the ID of the user the requested posts belong to
    pub async fn get_pinned_posts_by_user(&self, id: usize) -> Result<Vec<Post>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM posts WHERE owner = $1 AND context LIKE '%\"is_profile_pinned\":true%' ORDER BY created DESC",
            &[&(id as i64),],
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
            "SELECT * FROM posts WHERE replying_to = 0 AND NOT context LIKE '%\"is_nsfw\":true%' ORDER BY likes DESC, created ASC LIMIT $1 OFFSET $2",
            &[&(batch as i64), &((page * batch) as i64)],
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
                "SELECT * FROM posts WHERE (community = {} {query_string}) AND replying_to = 0 ORDER BY created DESC LIMIT $1 OFFSET $2",
                first.community
            ),
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_post_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("post".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get posts from all users the given user is following.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `batch` - the limit of posts in each page
    /// * `page` - the page number
    pub async fn get_posts_from_user_following(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Post>> {
        let following = self.get_userfollows_by_initiator_all(id).await?;
        let mut following = following.iter();
        let first = match following.next() {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut query_string: String = String::new();

        for user in following {
            query_string.push_str(&format!(" OR owner = {}", user.receiver));
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            &format!(
                "SELECT * FROM posts WHERE (owner = {} {query_string}) AND replying_to = 0 ORDER BY created DESC LIMIT $1 OFFSET $2",
                first.receiver
            ),
            &[&(batch as i64), &((page * batch) as i64)],
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
            CommunityWriteAccess::Owner => uid == community.owner,
            CommunityWriteAccess::Joined => {
                match self
                    .get_membership_by_owner_community(uid, community.id)
                    .await
                {
                    Ok(m) => !(!m.role.check_member()),
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
        // check values (if this isn't reposting something else)
        let is_reposting = if let Some(ref repost) = data.context.repost {
            repost.reposting.is_some()
        } else {
            false
        };

        if !is_reposting {
            if data.content.len() < 2 {
                return Err(Error::DataTooShort("content".to_string()));
            } else if data.content.len() > 4096 {
                return Err(Error::DataTooLong("content".to_string()));
            }
        }

        // check permission in community
        let community = match self.get_community_by_id(data.community).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        if !self.check_can_post(&community, data.owner).await {
            return Err(Error::NotAllowed);
        }

        // mirror nsfw state
        data.context.is_nsfw = community.context.is_nsfw;

        // check if we're reposting a post
        let reposting = if let Some(ref repost) = data.context.repost {
            if let Some(id) = repost.reposting {
                Some(self.get_post_by_id(id).await?)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ref rt) = reposting {
            data.context.reposts_enabled = false; // cannot repost reposts

            // make sure we aren't trying to repost a repost
            if if let Some(ref repost) = rt.context.repost {
                !(!repost.is_repost)
            } else {
                false
            } {
                return Err(Error::MiscError("Cannot repost a repost".to_string()));
            }

            // ...
            if !rt.context.reposts_enabled {
                return Err(Error::MiscError("Post has reposts disabled".to_string()));
            }

            // check blocked status
            if let Ok(_) = self
                .get_userblock_by_initiator_receiver(rt.owner, data.owner)
                .await
            {
                return Err(Error::NotAllowed);
            }
        }

        // check if the post we're replying to allows commments
        let replying_to = if let Some(id) = data.replying_to {
            Some(self.get_post_by_id(id).await?)
        } else {
            None
        };

        if let Some(ref rt) = replying_to {
            if !rt.context.comments_enabled {
                return Err(Error::MiscError("Post has comments disabled".to_string()));
            }

            // check blocked status
            if let Ok(_) = self
                .get_userblock_by_initiator_receiver(rt.owner, data.owner)
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
                            "[Somebody](/api/v1/auth/user/find/{}) mentioned you in their [post](/post/{}).",
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
                &format!("[@{username}](/api/v1/auth/user/find/{})", user.id),
            );
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
            params![
                &(data.id as i64),
                &(data.created as i64),
                &data.content,
                &(data.owner as i64),
                &(data.community as i64),
                &serde_json::to_string(&data.context).unwrap(),
                &if replying_to_id != "0" {
                    replying_to_id.parse::<i64>().unwrap()
                } else {
                    0_i64
                },
                &0_i32,
                &0_i32,
                &0_i32
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // incr comment count and send notification
        if let Some(rt) = replying_to {
            self.incr_post_comments(rt.id).await.unwrap();

            // send notification
            if data.owner != rt.owner {
                let owner = self.get_user_by_id(data.owner).await?;
                self.create_notification(Notification::new(
                    "Your post has received a new comment!".to_string(),
                    format!(
                        "[@{}](/api/v1/auth/user/find/{}) has commented on your [post](/post/{}).",
                        owner.username, owner.id, rt.id
                    ),
                    rt.owner,
                ))
                .await?;

                if !rt.context.comments_enabled {
                    return Err(Error::NotAllowed);
                }
            }
        }

        // return
        Ok(data.id)
    }

    pub async fn delete_post(&self, id: usize, user: User) -> Result<()> {
        let y = self.get_post_by_id(id).await?;

        let user_membership = self
            .get_membership_by_owner_community(user.id, y.community)
            .await?;

        if (user.id != y.owner)
            && !user_membership
                .role
                .check(CommunityPermission::MANAGE_POSTS)
        {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(AuditLogEntry::new(
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

        let res = execute!(&conn, "DELETE FROM posts WHERE id = $1", &[&(id as i64)]);

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

    pub async fn update_post_context(
        &self,
        id: usize,
        user: User,
        mut x: PostContext,
    ) -> Result<()> {
        let y = self.get_post_by_id(id).await?;
        x.repost = y.context.repost; // cannot change repost settings at all

        let user_membership = self
            .get_membership_by_owner_community(user.id, y.community)
            .await?;

        if (user.id != y.owner)
            && !user_membership
                .role
                .check(CommunityPermission::MANAGE_POSTS)
        {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(AuditLogEntry::new(
                    user.id,
                    format!("invoked `update_post_context` with x value `{id}`"),
                ))
                .await?
            }
        }

        // check if we can manage pins
        if x.is_pinned != y.context.is_pinned
            && !user_membership.role.check(CommunityPermission::MANAGE_PINS)
        {
            // lacking this permission is overtaken by having the MANAGE_POSTS
            // global permission
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(AuditLogEntry::new(
                    user.id,
                    format!("invoked `update_post_context(pinned)` with x value `{id}`"),
                ))
                .await?
            }
        }

        // check if we can manage profile pins
        if (x.is_profile_pinned != y.context.is_profile_pinned) && (user.id != y.owner) {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(AuditLogEntry::new(
                    user.id,
                    format!("invoked `update_post_context(profile_pinned)` with x value `{id}`"),
                ))
                .await?
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE posts SET context = $1 WHERE id = $2",
            params![&serde_json::to_string(&x).unwrap(), &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.post:{}", id)).await;

        // return
        Ok(())
    }

    pub async fn update_post_content(&self, id: usize, user: User, x: String) -> Result<()> {
        let mut y = self.get_post_by_id(id).await?;

        if user.id != y.owner {
            if !user.permissions.check(FinePermission::MANAGE_POSTS) {
                return Err(Error::NotAllowed);
            } else {
                self.create_audit_log_entry(AuditLogEntry::new(
                    user.id,
                    format!("invoked `update_post_content` with x value `{id}`"),
                ))
                .await?
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE posts SET content = $1 WHERE id = $2",
            params![&x, &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // update context
        y.context.edited = unix_epoch_timestamp() as usize;
        self.update_post_context(id, user, y.context).await?;

        // return
        Ok(())
    }

    auto_method!(incr_post_likes() -> "UPDATE posts SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(incr_post_dislikes() -> "UPDATE posts SET dislikes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_likes() -> "UPDATE posts SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
    auto_method!(decr_post_dislikes() -> "UPDATE posts SET dislikes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);

    auto_method!(incr_post_comments() -> "UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --incr);
    auto_method!(decr_post_comments() -> "UPDATE posts SET comment_count = comment_count - 1 WHERE id = $1" --cache-key-tmpl="atto.post:{}" --decr);
}
