use serde::{Serialize, Deserialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    /// A request to join a community.
    ///
    /// `users` table.
    CommunityJoin,
    /// A request to answer a question with a post.
    ///
    /// `questions` table.
    Answer,
}

#[derive(Serialize, Deserialize)]
pub struct ActionRequest {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub action_type: ActionType,
    /// The ID of the asset this request links to. Should exist in the correct
    /// table for the given [`ActionType`].
    pub linked_asset: usize,
}

impl ActionRequest {
    /// Create a new [`ActionRequest`].
    pub fn new(owner: usize, action_type: ActionType, linked_asset: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            owner,
            action_type,
            linked_asset,
        }
    }

    /// Create a new [`ActionRequest`] with the given `id`.
    pub fn with_id(id: usize, owner: usize, action_type: ActionType, linked_asset: usize) -> Self {
        Self {
            id,
            created: unix_epoch_timestamp() as usize,
            owner,
            action_type,
            linked_asset,
        }
    }
}
