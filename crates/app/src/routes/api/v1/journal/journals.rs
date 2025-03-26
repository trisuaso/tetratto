use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error, journal::Journal};

use crate::{
    State, get_user_from_token,
    routes::api::v1::{
        CreateJournal, UpdateJournalPrompt, UpdateJournalReadAccess, UpdateJournalTitle,
        UpdateJournalWriteAccess,
    },
};

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateJournal>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_page(Journal::new(req.title, req.prompt, user.id))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page created".to_string(),
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

    match data.delete_page(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page deleted".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_title_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalTitle>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_page_title(id, user, req.title).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_prompt_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalPrompt>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_page_prompt(id, user, req.prompt).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_read_access_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalReadAccess>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_page_read_access(id, user, req.access).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn update_write_access_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateJournalWriteAccess>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_page_write_access(id, user, req.access).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Page updated".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}
