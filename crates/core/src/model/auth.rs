use super::permissions::FinePermission;
use serde::{Deserialize, Serialize};
use tetratto_shared::{
    hash::{hash_salted, salt},
    snow::AlmostSnowflake,
    unix_epoch_timestamp,
};

/// `(ip, token, creation timestamp)`
pub type Token = (String, String, usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: usize,
    pub created: usize,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub settings: UserSettings,
    pub tokens: Vec<Token>,
    pub permissions: FinePermission,
    pub is_verified: bool,
    pub notification_count: usize,
    pub follower_count: usize,
    pub following_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub biography: String,
    #[serde(default)]
    pub private_profile: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            biography: String::new(),
            private_profile: false,
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new("<unknown>".to_string(), String::new())
    }
}

impl User {
    /// Create a new [`User`].
    pub fn new(username: String, password: String) -> Self {
        let salt = salt();
        let password = hash_salted(password, salt.clone());

        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            username,
            password,
            salt,
            settings: UserSettings::default(),
            tokens: Vec::new(),
            permissions: FinePermission::DEFAULT,
            is_verified: false,
            notification_count: 0,
            follower_count: 0,
            following_count: 0,
        }
    }

    /// Deleted user profile.
    pub fn deleted() -> Self {
        Self {
            username: "<deleted>".to_string(),
            id: 0,
            ..Default::default()
        }
    }

    /// Create a new token
    ///
    /// # Returns
    /// `(unhashed id, token)`
    pub fn create_token(ip: &str) -> (String, Token) {
        let unhashed = tetratto_shared::hash::uuid();
        (
            unhashed.clone(),
            (
                ip.to_string(),
                tetratto_shared::hash::hash(unhashed),
                unix_epoch_timestamp() as usize,
            ),
        )
    }

    /// Check if the given password is correct for the user.
    pub fn check_password(&self, against: String) -> bool {
        self.password == hash_salted(against, self.salt.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub content: String,
    pub owner: usize,
    pub read: bool,
}

impl Notification {
    /// Returns a new [`Notification`].
    pub fn new(title: String, content: String, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            title,
            content,
            owner,
            read: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserFollow {
    pub id: usize,
    pub created: usize,
    pub initiator: usize,
    pub receiver: usize,
}

impl UserFollow {
    /// Create a new [`UserFollow`].
    pub fn new(initiator: usize, receiver: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            initiator,
            receiver,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserBlock {
    pub id: usize,
    pub created: usize,
    pub initiator: usize,
    pub receiver: usize,
}

impl UserBlock {
    /// Create a new [`UserBlock`].
    pub fn new(initiator: usize, receiver: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            initiator,
            receiver,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IpBan {
    pub ip: String,
    pub created: usize,
    pub reason: String,
    pub moderator: usize,
}

impl IpBan {
    /// Create a new [`IpBan`].
    pub fn new(ip: String, moderator: usize, reason: String) -> Self {
        Self {
            ip,
            created: unix_epoch_timestamp() as usize,
            reason,
            moderator,
        }
    }
}
