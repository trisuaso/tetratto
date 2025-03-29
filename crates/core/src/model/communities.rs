use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

use super::communities_permissions::CommunityPermission;

#[derive(Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub context: CommunityContext,
    /// The ID of the owner of the community.
    pub owner: usize,
    /// Who can read the community page.
    pub read_access: CommunityReadAccess,
    /// Who can write to the community page (create posts belonging to it).
    ///
    /// The owner of the community page (and moderators) are the ***only*** people
    /// capable of removing posts.
    pub write_access: CommunityWriteAccess,
    // likes
    pub likes: isize,
    pub dislikes: isize,
    // counts
    pub member_count: usize,
}

impl Community {
    /// Create a new [`Community`].
    pub fn new(title: String, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            title: title.clone(),
            context: CommunityContext {
                display_name: title,
                ..Default::default()
            },
            owner,
            read_access: CommunityReadAccess::default(),
            write_access: CommunityWriteAccess::default(),
            likes: 0,
            dislikes: 0,
            member_count: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommunityContext {
    pub display_name: String,
    pub description: String,
}

impl Default for CommunityContext {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            description: String::new(),
        }
    }
}

/// Who can read a [`Community`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunityReadAccess {
    /// Everybody can view the community.
    Everybody,
    /// Only people with the link to the community.
    Unlisted,
    /// Only the owner of the community.
    Private,
}

impl Default for CommunityReadAccess {
    fn default() -> Self {
        Self::Everybody
    }
}

/// Who can write to a [`Community`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunityWriteAccess {
    /// Everybody.
    Everybody,
    /// Only people who joined the community can write to it.
    ///
    /// Memberships can be managed by the owner of the community.
    Joined,
    /// Only the owner of the community.
    Owner,
}

impl Default for CommunityWriteAccess {
    fn default() -> Self {
        Self::Joined
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommunityMembership {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub community: usize,
    pub role: CommunityPermission,
}

impl CommunityMembership {
    /// Create a new [`CommunityMembership`].
    pub fn new(owner: usize, community: usize, role: CommunityPermission) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            owner,
            community,
            role,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostContext {
    pub comments_enabled: bool,
}

impl Default for PostContext {
    fn default() -> Self {
        Self {
            comments_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub id: usize,
    pub created: usize,
    pub content: String,
    /// The ID of the owner of this post.
    pub owner: usize,
    /// The ID of the [`Community`] this post belongs to.
    pub community: usize,
    /// Extra information about the post.
    pub context: PostContext,
    /// The ID of the post this post is a comment on.
    pub replying_to: Option<usize>,
    pub likes: isize,
    pub dislikes: isize,
    pub comment_count: usize,
}

impl Post {
    /// Create a new [`Post`].
    pub fn new(
        content: String,
        community: usize,
        replying_to: Option<usize>,
        owner: usize,
    ) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            content,
            owner,
            community,
            context: PostContext::default(),
            replying_to,
            likes: 0,
            dislikes: 0,
            comment_count: 0,
        }
    }
}
