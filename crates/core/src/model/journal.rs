use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

use super::journal_permissions::JournalPermission;

#[derive(Serialize, Deserialize)]
pub struct JournalPage {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub prompt: String,
    /// The ID of the owner of the journal page.
    pub owner: usize,
    /// Who can read the journal page.
    pub read_access: JournalPageReadAccess,
    /// Who can write to the journal page (create journal entries belonging to it).
    ///
    /// The owner of the journal page (and moderators) are the ***only*** people
    /// capable of removing entries.
    pub write_access: JournalPageWriteAccess,
}

impl JournalPage {
    /// Create a new [`JournalPage`].
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
            read_access: JournalPageReadAccess::default(),
            write_access: JournalPageWriteAccess::default(),
        }
    }
}

/// Who can read a [`JournalPage`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalPageReadAccess {
    /// Everybody can view the journal page from the owner's profile.
    Everybody,
    /// Only people with the link to the journal page.
    Unlisted,
    /// Only the owner of the journal page.
    Private,
}

impl Default for JournalPageReadAccess {
    fn default() -> Self {
        Self::Everybody
    }
}

/// Who can write to a [`JournalPage`].
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalPageWriteAccess {
    /// Everybody (authenticated + anonymous users).
    Everybody,
    /// Authenticated users only.
    Authenticated,
    /// Only people who joined the journal page can write to it.
    ///
    /// Memberships can be managed by the owner of the journal page.
    Joined,
    /// Only the owner of the journal page.
    Owner,
}

impl Default for JournalPageWriteAccess {
    fn default() -> Self {
        Self::Authenticated
    }
}

#[derive(Serialize, Deserialize)]
pub struct JournalPageMembership {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub journal: usize,
    pub role: JournalPermission,
}

impl JournalPageMembership {
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
pub struct JournalEntryContext {
    pub comments_enabled: bool,
}

impl Default for JournalEntryContext {
    fn default() -> Self {
        Self {
            comments_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: usize,
    pub created: usize,
    pub content: String,
    /// The ID of the owner of this entry.
    pub owner: usize,
    /// The ID of the [`JournalPage`] this entry belongs to.
    pub journal: usize,
    /// Extra information about the journal entry.
    pub context: JournalEntryContext,
}

impl JournalEntry {
    /// Create a new [`JournalEntry`].
    pub fn new(content: String, journal: usize, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            content,
            owner,
            journal,
            context: JournalEntryContext::default(),
        }
    }
}
