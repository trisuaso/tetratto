pub mod auth;
pub mod journal;
pub mod reactions;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::Deserialize;
use tetratto_core::model::{
    journal::{JournalEntryContext, JournalPageReadAccess, JournalPageWriteAccess},
    reactions::AssetType,
};

pub fn routes() -> Router {
    Router::new()
        // reactions
        .route("/reactions", post(reactions::create_request))
        .route("/reactions/{id}", get(reactions::get_request))
        .route("/reactions/{id}", delete(reactions::delete_request))
        // journal pages
        .route("/pages", post(journal::pages::create_request))
        .route("/pages/{id}", delete(journal::pages::delete_request))
        .route(
            "/pages/{id}/title",
            post(journal::pages::update_title_request),
        )
        .route(
            "/pages/{id}/prompt",
            post(journal::pages::update_prompt_request),
        )
        .route(
            "/pages/{id}/access/read",
            post(journal::pages::update_read_access_request),
        )
        .route(
            "/pages/{id}/access/write",
            post(journal::pages::update_write_access_request),
        )
        // journal entries
        .route("/entries", post(journal::entries::create_request))
        .route("/entries/{id}", delete(journal::entries::delete_request))
        .route(
            "/entries/{id}/content",
            post(journal::entries::update_content_request),
        )
        .route(
            "/entries/{id}/context",
            post(journal::entries::update_context_request),
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
pub struct CreateJournalPage {
    pub title: String,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalPageTitle {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalPagePrompt {
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalPageReadAccess {
    pub access: JournalPageReadAccess,
}

#[derive(Deserialize)]
pub struct UpdateJournalPageWriteAccess {
    pub access: JournalPageWriteAccess,
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
    pub context: JournalEntryContext,
}

#[derive(Deserialize)]
pub struct CreateReaction {
    pub asset: usize,
    pub asset_type: AssetType,
    pub is_like: bool,
}
