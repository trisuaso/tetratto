use rainbeam_shared::{
    hash::{hash_salted, salt},
    snow::AlmostSnowflake,
    unix_epoch_timestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    DatabaseConnection(String),
    UserNotFound,
    RegistrationDisabled,
    DatabaseError,
    IncorrectPassword,
    AlreadyAuthenticated,
    Unknown,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::DatabaseConnection(msg) => msg.to_owned(),
            Error::UserNotFound => "Unable to find user with given parameters".to_string(),
            Error::RegistrationDisabled => "Registration is disabled".to_string(),
            Error::IncorrectPassword => "The given password is invalid".to_string(),
            Error::AlreadyAuthenticated => "Already authenticated".to_string(),
            _ => format!("An unknown error as occurred ({:?})", self),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// `(ip, token)`
pub type Token = (String, String);

#[derive(Debug)]
pub struct User {
    pub id: usize,
    pub created: usize,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub settings: UserSettings,
    pub tokens: Vec<Token>,
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
            tokens: vec![(String::new(), AlmostSnowflake::new(1234567890).to_string())],
        }
    }
}
