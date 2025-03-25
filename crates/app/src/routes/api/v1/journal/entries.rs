use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error, journal::JournalEntry};

use crate::{
    State, get_user_from_token,
    routes::api::v1::{CreateJournalEntry, UpdateJournalEntryContent, UpdateJournalEntryContext},
};

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateJournalEntry>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_entry(JournalEntry::new(req.content, req.journal, user.id))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Entry created".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn delete_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.delete_entry(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Entry deleted".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_content_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalEntryContent>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_entry_content(id, user, req.content).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Entry updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_context_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalEntryContext>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_entry_context(id, user, req.context).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Entry updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}
