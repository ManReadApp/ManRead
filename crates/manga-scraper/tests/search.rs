use std::path::Path;

use scraper_module::Attribute;
use scraper_testing::generate_tests;

async fn test_logic_no_query(uri: &str, _: &str) {
    let items = manga_scraper::init::register(Path::new("../../data/")).unwrap();
    let item = items.get_by_uri(uri).unwrap();
    let item = item.searchers.as_ref().unwrap();
    for page in 1..=2 {
        let data = item
            .search(scraper_module::SearchQuery::Simple(
                vec![
                    ("query".to_string(), Attribute::Str("".to_owned())),
                    ("page".to_string(), Attribute::Int(page)),
                    ("direction".to_string(), Attribute::Str("+".to_owned())),
                ]
                .into_iter()
                .collect(),
            ))
            .await
            .unwrap();
        println!("{:?}", data.items);
        assert!(data.items.len() > 0)
    }
}

async fn test_logic_query(uri: &str, content: &str) {
    let items = manga_scraper::init::register(Path::new("../../data/")).unwrap();
    let item = items.get_by_uri(uri).unwrap();
    let item = item.searchers.as_ref().unwrap();
    for (query, page) in content.lines().map(|v| {
        let (page, query) = v.split_once(" ").unwrap();
        (query.to_owned(), page.parse::<usize>().unwrap())
    }) {
        let data = item
            .search(scraper_module::SearchQuery::Simple(
                vec![
                    ("query".to_string(), Attribute::Str(query)),
                    ("page".to_string(), Attribute::Int(page as i64)),
                ]
                .into_iter()
                .collect(),
            ))
            .await
            .unwrap();
        println!("{:?}", data.items);
        assert!(data.items.len() > 0)
    }
}

// cargo test -p manga-scraper asura_search_test_no_query -- --nocapture
// cargo test -p manga-scraper asura_search_test_query -- --nocapture
//
generate_tests!((
    test_logic_no_query,
    "data/external",
    "search_test",
    "no_query"
));
generate_tests!((test_logic_query, "data/external", "search_test", "query"));
