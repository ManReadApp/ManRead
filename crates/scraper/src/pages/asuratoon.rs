pub fn get_first_url(base: &str, input: &str) -> Option<String> {
    let pattern = r#"<a\s+href="series/([^"]+)">"#;

    let regex = regex::Regex::new(pattern).expect("Failed to compile regex");

    if let Some(captures) = regex.captures(input) {
        if let Some(url) = captures.get(1) {
            return Some(format!("{base}/series/{}", url.as_str()));
        }
    }

    None
}
