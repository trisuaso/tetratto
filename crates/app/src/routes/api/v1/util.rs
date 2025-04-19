use super::auth::images::read_image;
use crate::State;
use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Extension,
};
use pathbufd::PathBufD;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProxyQuery {
    pub url: String,
}

/// Proxy an external url
pub async fn proxy_request(
    Query(props): Query<ProxyQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await);
    let http = &data.2;
    let data = &data.0;

    let image_url = &props.url;

    for host in &data.0.banned_hosts {
        if image_url.starts_with(host) {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(PathBufD::current().extend(&[
                    data.0.dirs.media.as_str(),
                    "images",
                    "default-banner.svg",
                ]))),
            );
        }
    }

    // get proxied image
    if image_url.is_empty() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-banner.svg",
            ]))),
        );
    }

    let guessed_mime = mime_guess::from_path(image_url)
        .first_raw()
        .unwrap_or("application/octet-stream");

    match http.get(image_url).send().await {
        Ok(stream) => {
            let size = stream.content_length();
            if size.unwrap_or_default() > 10485760 {
                // return defualt image (content too big)
                return (
                    [("Content-Type", "image/svg+xml")],
                    Body::from(read_image(PathBufD::current().extend(&[
                        data.0.dirs.media.as_str(),
                        "images",
                        "default-banner.svg",
                    ]))),
                );
            }

            if let Some(ct) = stream.headers().get("Content-Type") {
                let ct = ct.to_str().unwrap();
                let bad_ct = ["text/html", "text/plain"];
                if (!ct.starts_with("image/") && !ct.starts_with("font/")) | bad_ct.contains(&ct) {
                    // if we got html, return default banner (likely an error page)
                    return (
                        [("Content-Type", "image/svg+xml")],
                        Body::from(read_image(PathBufD::current().extend(&[
                            data.0.dirs.media.as_str(),
                            "images",
                            "default-banner.svg",
                        ]))),
                    );
                }
            }

            (
                [(
                    "Content-Type",
                    if guessed_mime == "text/html" {
                        "text/plain"
                    } else {
                        guessed_mime
                    },
                )],
                Body::from_stream(stream.bytes_stream()),
            )
        }
        Err(_) => (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-banner.svg",
            ]))),
        ),
    }
}

#[derive(Deserialize)]
pub struct LangFileQuery {
    #[serde(default)]
    pub id: String,
}

/// Set the current language.
pub async fn set_langfile_request(Query(props): Query<LangFileQuery>) -> impl IntoResponse {
    (
        {
            let mut headers = HeaderMap::new();

            headers.insert(
                "Set-Cookie",
                format!(
                    "__Secure-atto-lang={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
                    props.id,
                    60* 60 * 24 * 365
                )
                .parse()
                .unwrap(),
            );

            headers
        },
        "Language changed",
    )
}

pub async fn ip_test_request(
    headers: HeaderMap,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string()
}
