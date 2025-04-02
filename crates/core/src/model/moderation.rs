use serde::{Deserialize, Serialize};
use tetratto_shared::{snow::AlmostSnowflake, unix_epoch_timestamp};

use super::reactions::AssetType;

#[derive(Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: usize,
    pub created: usize,
    pub moderator: usize,
    pub content: String,
}

impl AuditLogEntry {
    /// Create a new [`AuditLogEntry`].
    pub fn new(moderator: usize, content: String) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            moderator,
            content,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Report {
    pub id: usize,
    pub created: usize,
    pub owner: usize,
    pub content: String,
    pub asset: usize,
    pub asset_type: AssetType,
}

impl Report {
    /// Create a new [`Report`].
    pub fn new(owner: usize, content: String, asset: usize, asset_type: AssetType) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            owner,
            content,
            asset,
            asset_type,
        }
    }
}
