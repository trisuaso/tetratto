pub mod auth;
pub mod communities;
pub mod reactions;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::Deserialize;
use tetratto_core::model::{
    communities::{CommunityContext, CommunityReadAccess, CommunityWriteAccess, PostContext},
    reactions::AssetType,
};

pub fn routes() -> Router {
    Router::new()
        // reactions
        .route("/reactions", post(reactions::create_request))
        .route("/reactions/{id}", get(reactions::get_request))
        .route("/reactions/{id}", delete(reactions::delete_request))
        // journal journals
        .route(
            "/communities",
            post(communities::communities::create_request),
        )
        .route(
            "/communities/{id}",
            delete(communities::communities::delete_request),
        )
        .route(
            "/communities/{id}/title",
            post(communities::communities::update_title_request),
        )
        .route(
            "/communities/{id}/context",
            post(communities::communities::update_context_request),
        )
        .route(
            "/journals/{id}/access/read",
            post(communities::communities::update_read_access_request),
        )
        .route(
            "/journals/{id}/access/write",
            post(communities::communities::update_write_access_request),
        )
        // posts
        .route("/posts", post(communities::posts::create_request))
        .route("/posts/{id}", delete(communities::posts::delete_request))
        .route(
            "/posts/{id}/content",
            post(communities::posts::update_content_request),
        )
        .route(
            "/posts/{id}/context",
            post(communities::posts::update_context_request),
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
        .route(
            "/auth/profile/{id}/follow",
            post(auth::social::follow_request),
        )
        .route(
            "/auth/profile/{id}/block",
            post(auth::social::block_request),
        )
        .route(
            "/auth/profile/{id}/settings",
            post(auth::profile::update_profile_settings_request),
        )
        .route(
            "/auth/profile/{id}/tokens",
            post(auth::profile::update_profile_tokens_request),
        )
        .route(
            "/auth/profile/{id}/verified",
            post(auth::profile::update_profile_is_verified_request),
        )
}

#[derive(Deserialize)]
pub struct AuthProps {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateCommunity {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalTitle {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateCommunityContext {
    pub context: CommunityContext,
}

#[derive(Deserialize)]
pub struct UpdateJournalReadAccess {
    pub access: CommunityReadAccess,
}

#[derive(Deserialize)]
pub struct UpdateJournalWriteAccess {
    pub access: CommunityWriteAccess,
}

#[derive(Deserialize)]
pub struct CreateJournalEntry {
    pub content: String,
    pub journal: usize,
    #[serde(default)]
    pub replying_to: Option<usize>,
}

#[derive(Deserialize)]
pub struct UpdateJournalEntryContent {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateJournalEntryContext {
    pub context: PostContext,
}

#[derive(Deserialize)]
pub struct CreateReaction {
    pub asset: usize,
    pub asset_type: AssetType,
    pub is_like: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserIsVerified {
    pub is_verified: bool,
}
