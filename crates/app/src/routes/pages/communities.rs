use super::render_error;
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::Error;

/// `/communities`
pub async fn list_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    let posts = match data.0.get_memberships_by_owner(user.id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("posts", &posts);

    // return
    Ok(Html(
        data.1
            .render("communities/list.html", &mut context)
            .unwrap(),
    ))
}
