use super::{PaginatedQuery, render_error};
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    Extension,
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::Error;
use std::fs::read_to_string;
use pathbufd::PathBufD;

pub async fn not_found(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);
    Html(
        render_error(
            Error::GeneralNotFound("page".to_string()),
            &jar,
            &data,
            &user,
        )
        .await,
    )
}

/// `/`
pub async fn index_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return {
                let lang = get_lang!(jar, data.0);
                let context = initial_context(&data.0.0, lang, &None).await;
                Html(data.1.render("misc/index.html", &context).unwrap())
            };
        }
    };

    let list = match data
        .0
        .get_posts_from_user_communities(user.id, 12, req.page)
        .await
    {
        Ok(l) => match data.0.fill_posts_with_community(l).await {
            Ok(l) => l,
            Err(e) => return Html(render_error(e, &jar, &data, &Some(user)).await),
        },
        Err(e) => return Html(render_error(e, &jar, &data, &Some(user)).await),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("list", &list);
    Html(data.1.render("timelines/home.html", &context).unwrap())
}

/// `/popular`
pub async fn popular_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let list = match data.0.get_popular_posts(12, req.page).await {
        Ok(l) => match data.0.fill_posts_with_community(l).await {
            Ok(l) => l,
            Err(e) => return Html(render_error(e, &jar, &data, &user).await),
        },
        Err(e) => return Html(render_error(e, &jar, &data, &user).await),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    context.insert("list", &list);
    Html(data.1.render("timelines/popular.html", &context).unwrap())
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
        data.1.render("misc/notifications.html", &context).unwrap(),
    ))
}

/// `/doc/{file_name}`
pub async fn markdown_document_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    if name.contains("//") | name.contains("..") | name.starts_with("/") {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &user).await,
        ));
    }

    let path = PathBufD::current().extend(&[&data.0.0.dirs.docs, &name]);
    let file = match read_to_string(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(Html(
                render_error(Error::MiscError(e.to_string()), &jar, &data, &user).await,
            ));
        }
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;
    context.insert("file", &file);
    context.insert("file_name", &name);

    // return
    Ok(Html(data.1.render("misc/markdown.html", &context).unwrap()))
}
