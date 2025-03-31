use super::render_error;
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::Error;

/// `/`
pub async fn index_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    Html(data.1.render("misc/index.html", &mut context).unwrap())
}

/// `/notifs`
pub async fn notifications_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    let notifications = match data.0.get_notifications_by_owner(user.id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("notifications", &notifications);

    // return
    Ok(Html(
        data.1
            .render("misc/notifications.html", &mut context)
            .unwrap(),
    ))
}
