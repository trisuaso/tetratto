use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error},
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::auth::{Notification, UserBlock, UserFollow};

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
        match data.delete_userfollow(userfollow.id, &user).await {
            Ok(_) => Json(ApiReturn {
                ok: true,
                message: "User unfollowed".to_string(),
                payload: (),
            }),
            Err(e) => Json(e.into()),
        }
    } else {
        // create
        match data.create_userfollow(UserFollow::new(user.id, id)).await {
            Ok(_) => {
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
            }
            Err(e) => Json(e.into()),
        }
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
                    match data.delete_userfollow(userfollow.id, &user).await {
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
                    match data.delete_userfollow(userfollow.id, &user).await {
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
