use std::path::Path;

#[test]
fn test_parsing_items() {
    let items = manga_scraper::init::register(Path::new("../../data/")).unwrap();
    assert!(items.len() > 0)
}
