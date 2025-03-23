use axum::{Extension, body::Body, extract::Path, response::IntoResponse};
use pathbufd::PathBufD;
use std::{
    fs::{File, exists},
    io::Read,
};

use crate::State;

pub fn read_image(path: PathBufD) -> Vec<u8> {
    let mut bytes = Vec::new();

    for byte in File::open(path).unwrap().bytes() {
        bytes.push(byte.unwrap())
    }

    bytes
}

/// Get a profile's avatar image
/// `/api/v1/auth/profile/{id}/avatar`
pub async fn avatar_request(
    Path(username): Path<String>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let user = match data.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(_) => {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(PathBufD::current().extend(&[
                    data.0.dirs.media.as_str(),
                    "images",
                    "default-avatar.svg",
                ]))),
            );
        }
    };

    let path =
        PathBufD::current().extend(&["avatars", &data.0.dirs.media, &format!("{}.avif", &user.id)]);

    if !exists(&path).unwrap() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-avatar.svg",
            ]))),
        );
    }

    (
        [("Content-Type", "image/avif")],
        Body::from(read_image(path)),
    )
}

/// Get a profile's banner image
/// `/api/v1/auth/profile/{id}/banner`
pub async fn banner_request(
    Path(username): Path<String>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let user = match data.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(_) => {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(PathBufD::current().extend(&[
                    data.0.dirs.media.as_str(),
                    "images",
                    "default-banner.svg",
                ]))),
            );
        }
    };

    let path =
        PathBufD::current().extend(&["avatars", &data.0.dirs.media, &format!("{}.avif", &user.id)]);

    if !exists(&path).unwrap() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-banner.svg",
            ]))),
        );
    }

    (
        [("Content-Type", "image/avif")],
        Body::from(read_image(path)),
    )
}
