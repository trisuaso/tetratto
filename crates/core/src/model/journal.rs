use serde::{Deserialize, Serialize};

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

/// Who can read a [`JournalPage`].
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub enum JournalPageWriteAccess {
    /// Everybody (authenticated + anonymous users).
    Everybody,
    /// Authenticated users only.
    Authenticated,
    /// Only the owner of the journal page.
    Owner,
}

impl Default for JournalPageWriteAccess {
    fn default() -> Self {
        Self::Authenticated
    }
}
