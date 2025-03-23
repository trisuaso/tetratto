use serde::{Deserialize, Serialize};
use tetratto_shared::{
    hash::{hash_salted, salt},
    snow::AlmostSnowflake,
    unix_epoch_timestamp,
};

use super::permissions::FinePermission;

/// `(ip, token, creation timestamp)`
pub type Token = (String, String, usize);

#[derive(Debug, Serialize)]
pub struct User {
    pub id: usize,
    pub created: usize,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub settings: UserSettings,
    pub tokens: Vec<Token>,
    pub permissions: FinePermission,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings;

impl Default for UserSettings {
    fn default() -> Self {
        Self {}
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
