pub fn get_first_url(input: &str) -> Option<&str> {
    let pattern = r#"<a\s+href="([^"]+)"\s+title="[^"]+">"#;

    let regex = regex::Regex::new(pattern).expect("Failed to compile regex");

    if let Some(captures) = regex.captures(input) {
        if let Some(url) = captures.get(1) {
            return Some(url.as_str());
        }
    }

    None
}
