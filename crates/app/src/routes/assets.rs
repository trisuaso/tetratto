use axum::response::IntoResponse;

macro_rules! serve_asset {
    ($fn_name:ident: $name:ident($type:literal)) => {
        pub async fn $fn_name() -> impl IntoResponse {
            ([("Content-Type", $type)], crate::assets::$name)
        }
    };
}

serve_asset!(favicon_request: FAVICON("image/svg+xml"));
serve_asset!(style_css_request: STYLE_CSS("text/css"));

serve_asset!(loader_js_request: LOADER_JS("text/javascript"));
serve_asset!(atto_js_request: ATTO_JS("text/javascript"));
serve_asset!(me_js_request: ME_JS("text/javascript"));
