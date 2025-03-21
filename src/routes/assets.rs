use axum::response::IntoResponse;

/// `/css/style.css`
pub async fn style_css_request() -> impl IntoResponse {
    (
        [("Content-Type", "text/css")],
        crate::data::assets::STYLE_CSS,
    )
}

/// `/js/atto.js`
pub async fn atto_js_request() -> impl IntoResponse {
    (
        [("Content-Type", "text/javascript")],
        crate::data::assets::ATTO_JS,
    )
}
