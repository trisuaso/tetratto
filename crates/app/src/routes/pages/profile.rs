use super::{PaginatedQuery, render_error};
use crate::{assets::initial_context, get_lang, get_user_from_token, sanitize::clean_settings, State};
use axum::{
    Extension,
    extract::{Path, Query},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tera::Context;
use tetratto_core::model::{Error, auth::User, communities::Community, permissions::FinePermission};

#[derive(Deserialize)]
pub struct SettingsProps {
    #[serde(default)]
    pub username: String,
}

/// `/settings`
pub async fn settings_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<SettingsProps>,
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

    let profile = if req.username.is_empty() | !user.permissions.check(FinePermission::MANAGE_USERS)
    {
        user.clone()
    } else {
        match data.0.get_user_by_username(&req.username).await {
            Ok(ua) => ua,
            Err(e) => {
                return Err(Html(render_error(e, &jar, &data, &None).await));
            }
        }
    };

    let tokens = profile.tokens.clone();

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("profile", &profile);
    context.insert("user_settings_serde", &clean_settings(&profile.settings));
    context.insert(
        "user_tokens_serde",
        &serde_json::to_string(&tokens)
            .unwrap()
            .replace("\"", "\\\""),
    );

    // return
    Ok(Html(
        data.1.render("profile/settings.html", &context).unwrap(),
    ))
}

pub fn profile_context(
    context: &mut Context,
    user: &Option<User>,
    profile: &User,
    communities: &Vec<Community>,
    is_self: bool,
    is_following: bool,
    is_following_you: bool,
    is_blocking: bool,
) {
    context.insert("profile", &profile);
    context.insert("communities", &communities);
    context.insert("is_self", &is_self);
    context.insert("is_following", &is_following);
    context.insert("is_following_you", &is_following_you);
    context.insert("is_blocking", &is_blocking);

    context.insert(
        "is_supporter",
        &profile.permissions.check(FinePermission::SUPPORTER),
    );

    if let Some(ua) = user {
        if !ua.settings.disable_other_themes | is_self {
            context.insert("use_user_theme", &false);
        }
    } else {
        context.insert("use_user_theme", &false);
    }
}

/// `/@{username}`
pub async fn posts_request(
    jar: CookieJar,
    Path(username): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let other_user = match data.0.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check if we're blocked
    if let Some(ref ua) = user {
        if data
            .0
            .get_userblock_by_initiator_receiver(other_user.id, ua.id)
            .await
            .is_ok()
        {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // check for private profile
    if other_user.settings.private_profile {
        if let Some(ref ua) = user {
            if (ua.id != other_user.id)
                && !ua.permissions.check(FinePermission::MANAGE_USERS)
                && data
                    .0
                    .get_userfollow_by_initiator_receiver(other_user.id, ua.id)
                    .await
                    .is_err()
            {
                return Err(Html(
                    render_error(Error::NotAllowed, &jar, &data, &user).await,
                ));
            }
        } else {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // fetch data
    let posts = match data
        .0
        .get_posts_by_user(other_user.id, 12, props.page)
        .await
    {
        Ok(p) => match data
            .0
            .fill_posts_with_community(p, if let Some(ref ua) = user { ua.id } else { 0 })
            .await
        {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let pinned = match data.0.get_pinned_posts_by_user(other_user.id).await {
        Ok(p) => match data
            .0
            .fill_posts_with_community(p, if let Some(ref ua) = user { ua.id } else { 0 })
            .await
        {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let communities = match data.0.get_memberships_by_owner(other_user.id).await {
        Ok(m) => match data.0.fill_communities(m).await {
            Ok(m) => m,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let is_self = if let Some(ref ua) = user {
        ua.id == other_user.id
    } else {
        false
    };

    let is_following = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_following_you = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_receiver_initiator(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_blocking = if let Some(ref ua) = user {
        data.0
            .get_userblock_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    context.insert("posts", &posts);
    context.insert("pinned", &pinned);
    context.insert("page", &props.page);
    profile_context(
        &mut context,
        &user,
        &other_user,
        &communities,
        is_self,
        is_following,
        is_following_you,
        is_blocking,
    );

    // return
    Ok(Html(data.1.render("profile/posts.html", &context).unwrap()))
}

/// `/@{username}/following`
pub async fn following_request(
    jar: CookieJar,
    Path(username): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let other_user = match data.0.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check if we're blocked
    if let Some(ref ua) = user {
        if data
            .0
            .get_userblock_by_initiator_receiver(other_user.id, ua.id)
            .await
            .is_ok()
        {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // check for private profile
    if other_user.settings.private_profile {
        if let Some(ref ua) = user {
            if ua.id != other_user.id
                && data
                    .0
                    .get_userfollow_by_initiator_receiver(other_user.id, ua.id)
                    .await
                    .is_err()
            {
                return Err(Html(
                    render_error(Error::NotAllowed, &jar, &data, &user).await,
                ));
            }
        } else {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // fetch data
    let list = match data
        .0
        .get_userfollows_by_initiator(other_user.id, 12, props.page)
        .await
    {
        Ok(l) => match data.0.fill_userfollows_with_receiver(l).await {
            Ok(l) => l,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let communities = match data.0.get_memberships_by_owner(other_user.id).await {
        Ok(m) => match data.0.fill_communities(m).await {
            Ok(m) => m,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let is_self = if let Some(ref ua) = user {
        ua.id == other_user.id
    } else {
        false
    };

    let is_following = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_following_you = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_receiver_initiator(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_blocking = if let Some(ref ua) = user {
        data.0
            .get_userblock_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    context.insert("list", &list);
    context.insert("page", &props.page);
    profile_context(
        &mut context,
        &user,
        &other_user,
        &communities,
        is_self,
        is_following,
        is_following_you,
        is_blocking,
    );

    // return
    Ok(Html(
        data.1.render("profile/following.html", &context).unwrap(),
    ))
}

/// `/@{username}/followers`
pub async fn followers_request(
    jar: CookieJar,
    Path(username): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let other_user = match data.0.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check if we're blocked
    if let Some(ref ua) = user {
        if data
            .0
            .get_userblock_by_initiator_receiver(other_user.id, ua.id)
            .await
            .is_ok()
        {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // check for private profile
    if other_user.settings.private_profile {
        if let Some(ref ua) = user {
            if ua.id != other_user.id
                && data
                    .0
                    .get_userfollow_by_initiator_receiver(other_user.id, ua.id)
                    .await
                    .is_err()
            {
                return Err(Html(
                    render_error(Error::NotAllowed, &jar, &data, &user).await,
                ));
            }
        } else {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // fetch data
    let list = match data
        .0
        .get_userfollows_by_receiver(other_user.id, 12, props.page)
        .await
    {
        Ok(l) => match data.0.fill_userfollows_with_initiator(l).await {
            Ok(l) => l,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let communities = match data.0.get_memberships_by_owner(other_user.id).await {
        Ok(m) => match data.0.fill_communities(m).await {
            Ok(m) => m,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let is_self = if let Some(ref ua) = user {
        ua.id == other_user.id
    } else {
        false
    };

    let is_following = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_following_you = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_receiver_initiator(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    let is_blocking = if let Some(ref ua) = user {
        data.0
            .get_userblock_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    context.insert("list", &list);
    context.insert("page", &props.page);
    profile_context(
        &mut context,
        &user,
        &other_user,
        &communities,
        is_self,
        is_following,
        is_following_you,
        is_blocking,
    );

    // return
    Ok(Html(
        data.1.render("profile/followers.html", &context).unwrap(),
    ))
}
