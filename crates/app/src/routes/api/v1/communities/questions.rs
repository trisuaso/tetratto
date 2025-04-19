use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{auth::IpBlock, communities::Question, ApiReturn, Error};
use crate::{get_user_from_token, routes::api::v1::CreateQuestion, State};

pub async fn create_request(
    jar: CookieJar,
    headers: HeaderMap,
    Extension(data): Extension<State>,
    Json(req): Json<CreateQuestion>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = get_user_from_token!(jar, data);

    if req.is_global && user.is_none() {
        return Json(Error::NotAllowed.into());
    }

    // get real ip
    let real_ip = headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string();

    // check for ip ban
    if data.get_ipban_by_ip(&real_ip).await.is_ok() {
        return Json(Error::NotAllowed.into());
    }

    // ...
    let mut props = Question::new(
        if let Some(ref ua) = user { ua.id } else { 0 },
        match req.receiver.parse::<usize>() {
            Ok(x) => x,
            Err(e) => return Json(Error::MiscError(e.to_string()).into()),
        },
        req.content,
        req.is_global,
        real_ip,
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

pub async fn ip_block_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    // get question
    let question = match data.get_question_by_id(id).await {
        Ok(q) => q,
        Err(e) => return Json(e.into()),
    };

    // check for an existing ip block
    if data
        .get_ipblock_by_initiator_receiver(user.id, &question.ip)
        .await
        .is_ok()
    {
        return Json(Error::NotAllowed.into());
    }

    // create ip block
    match data
        .create_ipblock(IpBlock::new(user.id, question.ip))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "IP blocked".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
