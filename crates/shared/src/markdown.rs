use ammonia::Builder;
use comrak::{Options, markdown_to_html};
use std::collections::HashSet;

/// Render markdown input into HTML
pub fn render_markdown(input: &str) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.superscript = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Option::Some(String::new());
    options.extension.tagfilter = true;
    options.render.unsafe_ = true;
    // options.render.escape = true;
    options.parse.smart = false;

    let html = markdown_to_html(input, &options);

    let mut allowed_attributes = HashSet::new();
    allowed_attributes.insert("id");
    allowed_attributes.insert("class");
    allowed_attributes.insert("ref");
    allowed_attributes.insert("aria-label");
    allowed_attributes.insert("lang");
    allowed_attributes.insert("title");
    allowed_attributes.insert("align");

    allowed_attributes.insert("data-color");
    allowed_attributes.insert("data-font-family");

    Builder::default()
        .generic_attributes(allowed_attributes)
        .clean(&html)
        .to_string()
        .replace(
            "src=\"",
            "loading=\"lazy\" src=\"/api/v1/util/ext/image?img=",
        )
        .replace("--&gt;", "<align class=\"right\">")
        .replace("-&gt;", "<align class=\"center\">")
        .replace("&lt;-", "</align>")
}
