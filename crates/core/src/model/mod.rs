pub mod auth;
pub mod journal;
pub mod permissions;

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
    Unknown,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
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
            _ => format!("An unknown error as occurred: ({:?})", self),
        }
    }
}

impl<T> Into<ApiReturn<T>> for Error
where
    T: Default + Serialize,
{
    fn into(self) -> ApiReturn<T> {
        ApiReturn {
            ok: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
