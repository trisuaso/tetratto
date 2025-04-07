use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

use super::communities_permissions::CommunityPermission;

#[derive(Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub context: CommunityContext,
    /// The ID of the owner of the community.
    pub owner: usize,
    /// Who can read the community.
    pub read_access: CommunityReadAccess,
    /// Who can write to the community (create posts belonging to it).
    ///
    /// The owner of the community (and moderators) are the ***only*** people
    /// capable of removing posts.
    pub write_access: CommunityWriteAccess,
    /// Who can join the community.
    pub join_access: CommunityJoinAccess,
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
            join_access: CommunityJoinAccess::default(),
            likes: 0,
            dislikes: 0,
            member_count: 0,
        }
    }

    /// Create the "void" community. This is where all posts with a deleted community
    /// resolve to.
    pub fn void() -> Self {
        Self {
            id: 0,
            created: 0,
            title: "void".to_string(),
            context: CommunityContext::default(),
            owner: 0,
            read_access: CommunityReadAccess::Joined,
            write_access: CommunityWriteAccess::Owner,
            join_access: CommunityJoinAccess::Nobody,
            likes: 0,
            dislikes: 0,
            member_count: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityContext {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_nsfw: bool,
}

impl Default for CommunityContext {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            description: String::new(),
            is_nsfw: false,
        }
    }
}

/// Who can read a [`Community`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunityReadAccess {
    /// Everybody can view the community.
    Everybody,
    /// Only people in the community can view the community.
    Joined,
}

impl Default for CommunityReadAccess {
    fn default() -> Self {
        Self::Everybody
    }
}

/// Who can write to a [`Community`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

/// Who can join a [`Community`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunityJoinAccess {
    /// Joins are closed. Nobody can join the community.
    Nobody,
    /// All authenticated users can join the community.
    Everybody,
    /// People must send a request to join.
    Request,
}

impl Default for CommunityJoinAccess {
    fn default() -> Self {
        Self::Everybody
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostContext {
    #[serde(default = "default_comments_enabled")]
    pub comments_enabled: bool,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub is_profile_pinned: bool,
    #[serde(default)]
    pub edited: usize,
    #[serde(default)]
    pub is_nsfw: bool,
}

fn default_comments_enabled() -> bool {
    true
}

impl Default for PostContext {
    fn default() -> Self {
        Self {
            comments_enabled: default_comments_enabled(),
            is_pinned: false,
            is_profile_pinned: false,
            edited: 0,
            is_nsfw: false,
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
