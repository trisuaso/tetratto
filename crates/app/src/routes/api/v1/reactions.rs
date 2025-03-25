use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error, reactions::Reaction};

use crate::{State, get_user_from_token, routes::api::v1::CreateReaction};

pub async fn get_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.get_reaction_by_owner_asset(user.id, id).await {
        Ok(r) => Json(ApiReturn {
            ok: true,
            message: "Reaction exists".to_string(),
            payload: Some(r),
        }),
        Err(e) => return Json(e.into()),
    }
}

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateReaction>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_reaction(Reaction::new(user.id, req.asset, req.asset_type))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Reaction created".to_string(),
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

    let reaction = match data.get_reaction_by_owner_asset(user.id, id).await {
        Ok(r) => r,
        Err(e) => return Json(e.into()),
    };

    match data.delete_reaction(reaction.id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Reaction deleted".to_string(),
            payload: (),
        }),
        Err(e) => return Json(e.into()),
    }
}
