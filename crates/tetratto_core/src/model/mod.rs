pub mod auth;
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
    DatabaseConnection(String),
    UserNotFound,
    RegistrationDisabled,
    DatabaseError(String),
    IncorrectPassword,
    AlreadyAuthenticated,
    DataTooLong(String),
    DataTooShort(String),
    Unknown,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::DatabaseConnection(msg) => msg.to_owned(),
            Error::DatabaseError(msg) => format!("Database error: {msg}"),
            Error::UserNotFound => "Unable to find user with given parameters".to_string(),
            Error::RegistrationDisabled => "Registration is disabled".to_string(),
            Error::IncorrectPassword => "The given password is invalid".to_string(),
            Error::AlreadyAuthenticated => "Already authenticated".to_string(),
            Error::DataTooLong(name) => format!("Given {name} is too long!"),
            Error::DataTooShort(name) => format!("Given {name} is too short!"),
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
