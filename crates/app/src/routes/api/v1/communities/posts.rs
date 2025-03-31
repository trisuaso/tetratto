use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{ApiReturn, Error, communities::Post};

use crate::{
    State, get_user_from_token,
    routes::api::v1::{CreatePost, UpdatePostContent, UpdatePostContext},
};

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreatePost>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_post(Post::new(
            req.content,
            match req.community.parse::<usize>() {
                Ok(x) => x,
                Err(e) => return Json(Error::MiscError(e.to_string()).into()),
            },
            if let Some(rt) = req.replying_to {
                match rt.parse::<usize>() {
                    Ok(x) => Some(x),
                    Err(e) => return Json(Error::MiscError(e.to_string()).into()),
                }
            } else {
                None
            },
            user.id,
        ))
        .await
    {
        Ok(id) => Json(ApiReturn {
            ok: true,
            message: "Post created".to_string(),
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

    match data.delete_post(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Post deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_content_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdatePostContent>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_post_content(id, user, req.content).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Post updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_context_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdatePostContext>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_post_context(id, user, req.context).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Post updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
