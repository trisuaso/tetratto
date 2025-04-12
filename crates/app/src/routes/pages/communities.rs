use super::{render_error, PaginatedQuery, SearchedQuery};
use crate::{assets::initial_context, get_lang, get_user_from_token, sanitize::clean_context, State};
use axum::{
    Extension,
    extract::{Path, Query},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use tera::Context;
use tetratto_core::model::{
    auth::User,
    communities::{Community, CommunityReadAccess},
    communities_permissions::CommunityPermission,
    permissions::FinePermission,
    Error,
};

macro_rules! check_permissions {
    ($community:ident, $jar:ident, $data:ident, $user:ident) => {{
        let mut is_member: bool = false;
        let mut can_manage_pins: bool = false;

        if let Some(ref ua) = $user {
            if let Ok(membership) = $data
                .0
                .get_membership_by_owner_community(ua.id, $community.id)
                .await
            {
                if membership.role.check_banned() {
                    return Err(Html(
                        render_error(Error::NotAllowed, &$jar, &$data, &$user).await,
                    ));
                } else if membership.role.check_member() {
                    is_member = true;
                }

                if membership.role.check(
                    tetratto_core::model::communities_permissions::CommunityPermission::MANAGE_PINS,
                ) {
                    can_manage_pins = true;
                }
            }
        }

        match $community.read_access {
            CommunityReadAccess::Joined => {
                if !is_member {
                    (false, can_manage_pins)
                } else {
                    (true, can_manage_pins)
                }
            }
            _ => (true, can_manage_pins),
        }
    }};
}

macro_rules! community_context_bools {
    ($data:ident, $user:ident, $community:ident) => {{
        let membership = if let Some(ref ua) = $user {
            match $data
                .0
                .get_membership_by_owner_community(ua.id, $community.id)
                .await
            {
                Ok(m) => Some(m),
                Err(_) => None,
            }
        } else {
            None
        };

        let is_owner = if let Some(ref ua) = $user {
            ua.id == $community.owner
        } else {
            false
        };

        let is_joined = if let Some(ref membership) = membership {
            membership.role.check_member()
        } else {
            false
        };

        let is_pending = if let Some(ref membership) = membership {
            membership.role.check(CommunityPermission::REQUESTED)
        } else {
            false
        };

        let can_post = if let Some(ref ua) = $user {
            $data.0.check_can_post(&$community, ua.id).await
        } else {
            false
        };

        let can_manage_posts = if let Some(ref membership) = membership {
            membership.role.check(CommunityPermission::MANAGE_POSTS)
        } else {
            false
        };

        let can_manage_community = if let Some(ref membership) = membership {
            membership.role.check(CommunityPermission::MANAGE_COMMUNITY)
        } else {
            false
        };

        let can_manage_roles = if let Some(ref membership) = membership {
            membership.role.check(CommunityPermission::MANAGE_ROLES)
        } else {
            false
        };

        (
            is_owner,
            is_joined,
            is_pending,
            can_post,
            can_manage_posts,
            can_manage_community,
            can_manage_roles,
        )
    }};
}

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

    let list = match data.0.get_memberships_by_owner(user.id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let popular_list = match data.0.get_popular_communities().await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let mut communities: Vec<Community> = Vec::new();
    for membership in &list {
        match data.0.get_community_by_id(membership.community).await {
            Ok(c) => communities.push(c),
            Err(e) => return Err(Html(render_error(e, &jar, &data, &None).await)),
        }
    }

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("list", &communities);
    context.insert("popular_list", &popular_list);

    // return
    Ok(Html(
        data.1.render("communities/list.html", &context).unwrap(),
    ))
}

/// `/communities/search`
pub async fn search_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<SearchedQuery>,
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

    let communities = match data
        .0
        .get_communities_searched(&req.text, 12, req.page)
        .await
    {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("list", &communities);
    context.insert("page", &req.page);
    context.insert("text", &req.text);

    // return
    Ok(Html(
        data.1.render("communities/search.html", &context).unwrap(),
    ))
}

/// `/communities/intents/post`
pub async fn create_post_request(
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

    let town_square = match data.0.get_community_by_id(data.0.0.town_square).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let memberships = match data.0.get_memberships_by_owner(user.id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let mut communities: Vec<Community> = Vec::new();
    for membership in memberships {
        if membership.community == data.0.0.town_square {
            // we already pulled the town square
            continue;
        }

        let community = match data.0.get_community_by_id(membership.community).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
        };

        communities.push(community)
    }

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("town_square", &town_square);
    context.insert("communities", &communities);

    // return
    Ok(Html(
        data.1
            .render("communities/create_post.html", &context)
            .unwrap(),
    ))
}

pub fn community_context(
    context: &mut Context,
    community: &Community,
    is_owner: bool,
    is_joined: bool,
    is_pending: bool,
    can_post: bool,
    can_read: bool,
    can_manage_posts: bool,
    can_manage_community: bool,
    can_manage_roles: bool,
) {
    context.insert("community", &community);
    context.insert("is_owner", &is_owner);
    context.insert("is_joined", &is_joined);
    context.insert("is_pending", &is_pending);
    context.insert("can_post", &can_post);
    context.insert("can_read", &can_read);
    context.insert("can_manage_posts", &can_manage_posts);
    context.insert("can_manage_community", &can_manage_community);
    context.insert("can_manage_roles", &can_manage_roles);
}

/// `/community/{title}`
pub async fn feed_request(
    jar: CookieJar,
    Path(title): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let community = match data.0.get_community_by_title(&title.to_lowercase()).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    if community.id == 0 {
        // don't show page for void community
        return Err(Html(
            render_error(
                Error::GeneralNotFound("community".to_string()),
                &jar,
                &data,
                &user,
            )
            .await,
        ));
    }

    // check permissions
    let (can_read, _) = check_permissions!(community, jar, data, user);

    // ...
    let feed = match data
        .0
        .get_posts_by_community(community.id, 12, props.page)
        .await
    {
        Ok(p) => match data.0.fill_posts(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let pinned = match data.0.get_pinned_posts_by_community(community.id).await {
        Ok(p) => match data.0.fill_posts(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let (
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    ) = community_context_bools!(data, user, community);

    context.insert("feed", &feed);
    context.insert("pinned", &pinned);
    context.insert("page", &props.page);
    community_context(
        &mut context,
        &community,
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_read,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    );

    // return
    Ok(Html(
        data.1.render("communities/feed.html", &context).unwrap(),
    ))
}

/// `/community/{id}/manage`
pub async fn settings_request(
    jar: CookieJar,
    Path(id): Path<usize>,
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

    let community = match data.0.get_community_by_id_no_void(id).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let membership = match data
        .0
        .get_membership_by_owner_community(user.id, community.id)
        .await
    {
        Ok(m) => m,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    if user.id != community.owner
        && !user.permissions.check(FinePermission::MANAGE_COMMUNITIES)
        && !membership.role.check(CommunityPermission::MANAGE_COMMUNITY)
    {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("community", &community);
    context.insert(
        "community_context_serde",
        &clean_context(&community.context),
    );

    // return
    Ok(Html(
        data.1
            .render("communities/settings.html", &context)
            .unwrap(),
    ))
}

/// `/post/{id}`
pub async fn post_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let post = match data.0.get_post_by_id(id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let community = match data.0.get_community_by_id(post.community).await {
        Ok(c) => c,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check repost
    let reposting = if let Some(ref repost) = post.context.repost {
        if let Some(reposting) = repost.reposting {
            let mut x = match data.0.get_post_by_id(reposting).await {
                Ok(p) => p,
                Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
            };

            x.mark_as_repost();
            Some((
                match data.0.get_user_by_id(x.owner).await {
                    Ok(ua) => ua,
                    Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
                },
                x,
            ))
        } else {
            None
        }
    } else {
        None
    };

    // check permissions
    let (can_read, can_manage_pins) = check_permissions!(community, jar, data, user);

    // ...
    let feed = match data.0.get_post_comments(post.id, 12, props.page).await {
        Ok(p) => match data.0.fill_posts(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let (
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    ) = community_context_bools!(data, user, community);

    context.insert("post", &post);
    context.insert("reposting", &reposting);
    context.insert("replies", &feed);
    context.insert("page", &props.page);
    context.insert(
        "owner",
        &data
            .0
            .get_user_by_id(post.owner)
            .await
            .unwrap_or(User::deleted()),
    );
    context.insert(
        "post_context_serde",
        &serde_json::to_string(&post.context)
            .unwrap()
            .replace("\"", "\\\""),
    );
    context.insert("can_manage_pins", &can_manage_pins);

    community_context(
        &mut context,
        &community,
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_read,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    );

    // return
    Ok(Html(
        data.1.render("communities/post.html", &context).unwrap(),
    ))
}

/// `/community/{title}/members`
pub async fn members_request(
    jar: CookieJar,
    Path(title): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let community = match data.0.get_community_by_title(&title.to_lowercase()).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    if community.id == 0 {
        // don't show page for void community
        return Err(Html(
            render_error(
                Error::GeneralNotFound("community".to_string()),
                &jar,
                &data,
                &user,
            )
            .await,
        ));
    }

    // check permissions
    let (can_read, _) = check_permissions!(community, jar, data, user);

    // ...
    let list = match data
        .0
        .get_memberships_by_community(community.id, community.owner, 12, props.page)
        .await
    {
        Ok(p) => match data.0.fill_users(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // get community owner
    let owner = match data.0.get_user_by_id(community.owner).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let (
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    ) = community_context_bools!(data, user, community);

    context.insert("list", &list);
    context.insert("page", &props.page);
    context.insert("owner", &owner);
    community_context(
        &mut context,
        &community,
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_read,
        can_manage_posts,
        can_manage_community,
        can_manage_roles,
    );

    // return
    Ok(Html(
        data.1.render("communities/members.html", &context).unwrap(),
    ))
}
