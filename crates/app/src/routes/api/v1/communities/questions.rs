use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{communities::Question, ApiReturn, Error};
use crate::{get_user_from_token, routes::api::v1::CreateQuestion, State};

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateQuestion>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let mut props = Question::new(
        user.id,
        match req.receiver.parse::<usize>() {
            Ok(x) => x,
            Err(e) => return Json(Error::MiscError(e.to_string()).into()),
        },
        req.content,
        req.is_global,
    );

    if !req.community.is_empty() {
        props.is_global = true;
        props.receiver = 0;
        props.community = match req.community.parse::<usize>() {
            Ok(x) => x,
            Err(e) => return Json(Error::MiscError(e.to_string()).into()),
        }
    }

    match data.create_question(props).await {
        Ok(id) => Json(ApiReturn {
            ok: true,
            message: "Question created".to_string(),
            payload: Some(id.to_string()),
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

    match data.delete_question(id, &user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Question deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
