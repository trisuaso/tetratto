use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

use super::journal_permissions::JournalPermission;

#[derive(Serialize, Deserialize)]
pub struct Journal {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub prompt: String,
    /// The ID of the owner of the journal page.
    pub owner: usize,
    /// Who can read the journal page.
    pub read_access: JournalReadAccess,
    /// Who can write to the journal page (create journal entries belonging to it).
    ///
    /// The owner of the journal page (and moderators) are the ***only*** people
    /// capable of removing entries.
    pub write_access: JournalWriteAccess,
    pub likes: isize,
    pub dislikes: isize,
}

impl Journal {
    /// Create a new [`Journal`].
    pub fn new(title: String, prompt: String, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            title,
            prompt,
            owner,
            read_access: JournalReadAccess::default(),
            write_access: JournalWriteAccess::default(),
            likes: 0,
            dislikes: 0,
        }
    }
}

/// Who can read a [`Journal`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalReadAccess {
    /// Everybody can view the journal page from the owner's profile.
    Everybody,
    /// Only people with the link to the journal page.
    Unlisted,
    /// Only the owner of the journal page.
    Private,
}

impl Default for JournalReadAccess {
    fn default() -> Self {
        Self::Everybody
    }
}

/// Who can write to a [`Journal`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalWriteAccess {
    /// Everybody (authenticated users only still).
    Everybody,
    /// Only people who joined the journal page can write to it.
    ///
    /// Memberships can be managed by the owner of the journal page.
    Joined,
    /// Only the owner of the journal page.
    Owner,
}

impl Default for JournalWriteAccess {
    fn default() -> Self {
        Self::Joined
    }
}

#[derive(Serialize, Deserialize)]
pub struct JournalMembership {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub journal: usize,
    pub role: JournalPermission,
}

impl JournalMembership {
    pub fn new(owner: usize, journal: usize, role: JournalPermission) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            owner,
            journal,
            role,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JournalPostContext {
    pub comments_enabled: bool,
}

impl Default for JournalPostContext {
    fn default() -> Self {
        Self {
            comments_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JournalPost {
    pub id: usize,
    pub created: usize,
    pub content: String,
    /// The ID of the owner of this entry.
    pub owner: usize,
    /// The ID of the [`Journal`] this entry belongs to.
    pub journal: usize,
    /// Extra information about the journal entry.
    pub context: JournalPostContext,
    /// The ID of the post this post is a comment on.
    pub replying_to: Option<usize>,
    pub likes: isize,
    pub dislikes: isize,
    pub comment_count: usize,
}

impl JournalPost {
    /// Create a new [`JournalEntry`].
    pub fn new(content: String, journal: usize, replying_to: Option<usize>, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            content,
            owner,
            journal,
            context: JournalPostContext::default(),
            replying_to,
            likes: 0,
            dislikes: 0,
            comment_count: 0,
        }
    }
}
