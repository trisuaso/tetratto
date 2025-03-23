use axum::response::IntoResponse;

/// `/css/style.css`
pub async fn style_css_request() -> impl IntoResponse {
    ([("Content-Type", "text/css")], crate::assets::STYLE_CSS)
}

/// `/js/atto.js`
pub async fn atto_js_request() -> impl IntoResponse {
    (
        [("Content-Type", "text/javascript")],
        crate::assets::ATTO_JS,
    )
}

/// `/js/atto.js`
pub async fn loader_js_request() -> impl IntoResponse {
    (
        [("Content-Type", "text/javascript")],
        crate::assets::LOADER_JS,
    )
}
