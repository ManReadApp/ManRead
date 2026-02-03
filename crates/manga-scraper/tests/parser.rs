use manga_scraper::init::parse::selectors;

#[test]
fn test_parser() {
    let v = selectors::parse(include_str!("parser.txt")).unwrap();
    assert!(v.len() > 0)
}
