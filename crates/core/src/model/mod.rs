pub mod auth;
pub mod communities;
pub mod communities_permissions;
pub mod moderation;
pub mod permissions;
pub mod reactions;
pub mod requests;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiReturn<T>
where
    T: Serialize,
{
    pub ok: bool,
    pub message: String,
    pub payload: T,
}

#[derive(Debug)]
pub enum Error {
    MiscError(String),
    DatabaseConnection(String),
    UserNotFound,
    GeneralNotFound(String),
    RegistrationDisabled,
    DatabaseError(String),
    IncorrectPassword,
    NotAllowed,
    AlreadyAuthenticated,
    DataTooLong(String),
    DataTooShort(String),
    UsernameInUse,
    TitleInUse,
    QuestionsDisabled,
    Unknown,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::MiscError(msg) => msg.to_owned(),
            Self::DatabaseConnection(msg) => msg.to_owned(),
            Self::DatabaseError(msg) => format!("Database error: {msg}"),
            Self::UserNotFound => "Unable to find user with given parameters".to_string(),
            Self::GeneralNotFound(name) => format!("Unable to find requested {name}"),
            Self::RegistrationDisabled => "Registration is disabled".to_string(),
            Self::IncorrectPassword => "The given password is invalid".to_string(),
            Self::NotAllowed => "You are not allowed to do this".to_string(),
            Self::AlreadyAuthenticated => "Already authenticated".to_string(),
            Self::DataTooLong(name) => format!("Given {name} is too long!"),
            Self::DataTooShort(name) => format!("Given {name} is too short!"),
            Self::UsernameInUse => "Username in use".to_string(),
            Self::TitleInUse => "Title in use".to_string(),
            Self::QuestionsDisabled => "You are not allowed to ask questions there".to_string(),
            _ => format!("An unknown error as occurred: ({:?})", self),
        })
    }
}

impl<T> From<Error> for ApiReturn<T>
where
    T: Default + Serialize,
{
    fn from(val: Error) -> Self {
        ApiReturn {
            ok: false,
            message: val.to_string(),
            payload: T::default(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
