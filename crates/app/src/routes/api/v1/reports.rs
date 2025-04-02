use super::CreateReport;
use crate::{State, get_user_from_token};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error, moderation::Report};

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateReport>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let asset_id = match req.asset.parse::<usize>() {
        Ok(n) => n,
        Err(e) => return Json(Error::MiscError(e.to_string()).into()),
    };

    match data
        .create_report(Report::new(user.id, req.content, asset_id, req.asset_type))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Report created".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
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

    match data.delete_report(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Report deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
