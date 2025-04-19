use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error},
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::auth::{FollowResult, IpBlock, Notification, UserBlock, UserFollow};

/// Toggle following on the given user.
pub async fn follow_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if let Ok(userfollow) = data.get_userfollow_by_initiator_receiver(user.id, id).await {
        // delete
        match data.delete_userfollow(userfollow.id, &user, false).await {
            Ok(_) => Json(ApiReturn {
                ok: true,
                message: "User unfollowed".to_string(),
                payload: (),
            }),
            Err(e) => Json(e.into()),
        }
    } else {
        // create
        match data
            .create_userfollow(UserFollow::new(user.id, id), false)
            .await
        {
            Ok(r) => {
                if r == FollowResult::Followed {
                    if let Err(e) = data
                        .create_notification(Notification::new(
                            "Somebody has followed you!".to_string(),
                            format!(
                                "You have been followed by [@{}](/api/v1/auth/user/find/{}).",
                                user.username, user.id
                            ),
                            id,
                        ))
                        .await
                    {
                        return Json(e.into());
                    };

                    Json(ApiReturn {
                        ok: true,
                        message: "User followed".to_string(),
                        payload: (),
                    })
                } else {
                    Json(ApiReturn {
                        ok: true,
                        message: "Asked to follow user".to_string(),
                        payload: (),
                    })
                }
            }
            Err(e) => Json(e.into()),
        }
    }
}

pub async fn cancel_follow_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.delete_request(user.id, id, &user, true).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Follow request deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn accept_follow_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    // delete the request
    if let Err(e) = data.delete_request(id, user.id, &user, true).await {
        return Json(e.into());
    }

    // create follow
    match data
        .create_userfollow(UserFollow::new(id, user.id), true)
        .await
    {
        Ok(_) => {
            if let Err(e) = data
                .create_notification(Notification::new(
                    "Somebody has accepted your follow request!".to_string(),
                    format!(
                        "You are now following [@{}](/api/v1/auth/user/find/{}).",
                        user.username, user.id
                    ),
                    id,
                ))
                .await
            {
                return Json(e.into());
            };

            Json(ApiReturn {
                ok: true,
                message: "User follow request accepted".to_string(),
                payload: (),
            })
        }
        Err(e) => Json(e.into()),
    }
}

/// Toggle blocking on the given user.
pub async fn block_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if let Ok(userblock) = data.get_userblock_by_initiator_receiver(user.id, id).await {
        // delete
        match data.delete_userblock(userblock.id, user).await {
            Ok(_) => Json(ApiReturn {
                ok: true,
                message: "User unblocked".to_string(),
                payload: (),
            }),
            Err(e) => Json(e.into()),
        }
    } else {
        // create
        match data.create_userblock(UserBlock::new(user.id, id)).await {
            Ok(_) => {
                if let Ok(userfollow) = data.get_userfollow_by_initiator_receiver(user.id, id).await
                {
                    // automatically unfollow
                    match data.delete_userfollow(userfollow.id, &user, false).await {
                        Ok(_) => Json(ApiReturn {
                            ok: true,
                            message: "User blocked".to_string(),
                            payload: (),
                        }),
                        Err(e) => Json(e.into()),
                    }
                } else if let Ok(userfollow) =
                    data.get_userfollow_by_receiver_initiator(user.id, id).await
                {
                    // automatically unfollow
                    match data.delete_userfollow(userfollow.id, &user, false).await {
                        Ok(_) => Json(ApiReturn {
                            ok: true,
                            message: "User blocked".to_string(),
                            payload: (),
                        }),
                        Err(e) => Json(e.into()),
                    }
                } else {
                    // not following user, don't do anything else
                    Json(ApiReturn {
                        ok: true,
                        message: "User blocked".to_string(),
                        payload: (),
                    })
                }
            }
            Err(e) => Json(e.into()),
        }
    }
}

/// Toggle IP blocking on the given IP.
pub async fn ip_block_request(
    jar: CookieJar,
    Path(ip): Path<String>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if let Ok(ipblock) = data.get_ipblock_by_initiator_receiver(user.id, &ip).await {
        // delete
        match data.delete_ipblock(ipblock.id, user).await {
            Ok(_) => Json(ApiReturn {
                ok: true,
                message: "IP unblocked".to_string(),
                payload: (),
            }),
            Err(e) => Json(e.into()),
        }
    } else {
        // create
        match data.create_ipblock(IpBlock::new(user.id, ip)).await {
            Ok(_) => Json(ApiReturn {
                ok: true,
                message: "IP blocked".to_string(),
                payload: (),
            }),
            Err(e) => Json(e.into()),
        }
    }
}
