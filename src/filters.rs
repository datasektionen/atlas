#[askama::filter_fn]
pub fn parse_markdown(value: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    // The docs specify that GFM cannot error and that unwrapping is fine.
    Ok(markdown::to_html_with_options(value, &markdown::Options::gfm()).unwrap())
}
