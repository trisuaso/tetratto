use crate::{State, get_user_from_token};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error};

pub async fn delete_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path((id, linked_asset)): Path<(usize, usize)>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.delete_request(id, linked_asset, &user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Request deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn delete_all_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.delete_all_requests(&user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Requests cleared".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
