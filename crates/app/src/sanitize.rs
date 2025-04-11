use ammonia::Builder;
use tetratto_core::model::{auth::UserSettings, communities::CommunityContext};

/// Escape profile colors
pub fn color_escape(color: &str) -> String {
    remove_tags(
        &color
            .replace(";", "")
            .replace("<", "&lt;")
            .replace(">", "%gt;")
            .replace("}", "")
            .replace("{", "")
            .replace("url(\"", "url(\"/api/v0/util/ext/image?img=")
            .replace("url('", "url('/api/v0/util/ext/image?img=")
            .replace("url(https://", "url(/api/v0/util/ext/image?img=https://"),
    )
}

/// Clean profile metadata
pub fn remove_tags(input: &str) -> String {
    Builder::default()
        .rm_tags(&["img", "a", "span", "p", "h1", "h2", "h3", "h4", "h5", "h6"])
        .clean(input)
        .to_string()
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("</script>", "</not-script")
}

fn clean_single(input: &str) -> String {
    input
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("url(\"", "url(\"/api/v0/util/ext/image?img=")
        .replace("url(https://", "url(/api/v0/util/ext/image?img=https://")
        .replace("<style>", "")
        .replace("</style>", "")
}

/// Clean user settings
pub fn clean_settings(settings: &UserSettings) -> String {
    remove_tags(&serde_json::to_string(&clean_settings_raw(settings)).unwrap())
        .replace("\u{200d}", "")
        // how do you end up with these in your settings?
        .replace("\u{0010}", "")
        .replace("\u{0011}", "")
        .replace("\u{0012}", "")
        .replace("\u{0013}", "")
        .replace("\u{0014}", "")
}

/// Clean user settings row
pub fn clean_settings_raw(settings: &UserSettings) -> UserSettings {
    let mut settings = settings.to_owned();

    settings.biography = clean_single(&settings.biography);
    settings.theme_hue = clean_single(&settings.theme_hue);
    settings.theme_sat = clean_single(&settings.theme_sat);
    settings.theme_lit = clean_single(&settings.theme_lit);

    settings
}

/// Clean community context
pub fn clean_context(context: &CommunityContext) -> String {
    remove_tags(&serde_json::to_string(&clean_context_raw(context)).unwrap())
        .replace("\u{200d}", "")
        // how do you end up with these in your settings?
        .replace("\u{0010}", "")
        .replace("\u{0011}", "")
        .replace("\u{0012}", "")
        .replace("\u{0013}", "")
        .replace("\u{0014}", "")
}

/// Clean community context row
pub fn clean_context_raw(context: &CommunityContext) -> CommunityContext {
    let mut context = context.to_owned();
    context.description = clean_single(&context.description);
    context
}
