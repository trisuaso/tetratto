use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

/// All of the items which support reactions.
#[derive(Serialize, Deserialize)]
pub enum AssetType {
    #[serde(alias = "community")]
    Community,
    #[serde(alias = "post")]
    Post,
    #[serde(alias = "question")]
    Question,
    #[serde(alias = "user")]
    User,
}

#[derive(Serialize, Deserialize)]
pub struct Reaction {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub asset: usize,
    pub asset_type: AssetType,
    pub is_like: bool,
}

impl Reaction {
    /// Create a new [`Reaction`].
    pub fn new(owner: usize, asset: usize, asset_type: AssetType, is_like: bool) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            owner,
            asset,
            asset_type,
            is_like,
        }
    }
}
