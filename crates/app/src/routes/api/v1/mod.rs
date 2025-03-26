pub mod auth;
pub mod journal;
pub mod reactions;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::Deserialize;
use tetratto_core::model::{
    journal::{JournalPostContext, JournalReadAccess, JournalWriteAccess},
    reactions::AssetType,
};

pub fn routes() -> Router {
    Router::new()
        // reactions
        .route("/reactions", post(reactions::create_request))
        .route("/reactions/{id}", get(reactions::get_request))
        .route("/reactions/{id}", delete(reactions::delete_request))
        // journal journals
        .route("/journals", post(journal::journals::create_request))
        .route("/journals/{id}", delete(journal::journals::delete_request))
        .route(
            "/journals/{id}/title",
            post(journal::journals::update_title_request),
        )
        .route(
            "/journals/{id}/prompt",
            post(journal::journals::update_prompt_request),
        )
        .route(
            "/journals/{id}/access/read",
            post(journal::journals::update_read_access_request),
        )
        .route(
            "/journals/{id}/access/write",
            post(journal::journals::update_write_access_request),
        )
        // journal posts
        .route("/posts", post(journal::posts::create_request))
        .route("/posts/{id}", delete(journal::posts::delete_request))
        .route(
            "/posts/{id}/content",
            post(journal::posts::update_content_request),
        )
        .route(
            "/posts/{id}/context",
            post(journal::posts::update_context_request),
        )
        // auth
        // global
        .route("/auth/register", post(auth::register_request))
        .route("/auth/login", post(auth::login_request))
        .route("/auth/logout", post(auth::logout_request))
        .route(
            "/auth/upload/avatar",
            post(auth::images::upload_avatar_request),
        )
        .route(
            "/auth/upload/banner",
            post(auth::images::upload_banner_request),
        )
        // profile
        .route(
            "/auth/profile/{id}/avatar",
            get(auth::images::avatar_request),
        )
        .route(
            "/auth/profile/{id}/banner",
            get(auth::images::banner_request),
        )
}

#[derive(Deserialize)]
pub struct AuthProps {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateJournal {
    pub title: String,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalTitle {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalPrompt {
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalReadAccess {
    pub access: JournalReadAccess,
}

#[derive(Deserialize)]
pub struct UpdateJournalWriteAccess {
    pub access: JournalWriteAccess,
}

#[derive(Deserialize)]
pub struct CreateJournalEntry {
    pub content: String,
    pub journal: usize,
}

#[derive(Deserialize)]
pub struct UpdateJournalEntryContent {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalEntryContext {
    pub context: JournalPostContext,
}

#[derive(Deserialize)]
pub struct CreateReaction {
    pub asset: usize,
    pub asset_type: AssetType,
    pub is_like: bool,
}
