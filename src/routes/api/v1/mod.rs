pub mod auth;
use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router {
    Router::new().route("/auth/register", post(auth::register_request))
}

#[derive(Serialize, Deserialize)]
pub struct ApiReturn<T>
where
    T: Serialize,
{
    pub ok: bool,
    pub message: String,
    pub payload: T,
}

impl<T> ApiReturn<T>
where
    T: Serialize,
{
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Deserialize)]
pub struct AuthProps {
    pub username: String,
    pub password: String,
}
