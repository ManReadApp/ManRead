use std::path::Path;

use scraper_testing::generate_tests;

async fn test_logic(_: &str, content: &str) {
    let services = manga_scraper::init::register(Path::new("../../data/")).unwrap();

    let items = content.split("\n\n").map(|v| v.trim()).collect::<Vec<_>>();
    for item in items {
        let service = services.get(item).unwrap();
        let reader = service.reader.as_ref().unwrap();
        match reader.multi(item) {
            scraper_module::Mode::Single => {
                let first = reader.scrape_pages(&item).await.unwrap();
                println!("{:#?}", first);
                assert!(first.len() > 1, "Expected more than one page");
            }
            scraper_module::Mode::Multi => {
                let chapters = reader.scrape_chapters(item).await.unwrap();
                println!("{:?}", chapters);
                assert!(chapters.len() > 1, "Expected more than one chapter");
                let first = chapters.first().unwrap();
                let last = chapters.last().unwrap();
                let first = reader.scrape_pages(&first.url).await.unwrap();
                println!("{:#?}", first);
                assert!(first.len() > 1, "Expected more than one page");
                let last = reader.scrape_pages(&last.url).await.unwrap();
                println!("{:#?}", last);
                assert!(last.len() > 1, "Expected more than one page");
            }
            scraper_module::Mode::TextMulti => todo!(),
            scraper_module::Mode::TextSingle => todo!(),
        }
    }
}

generate_tests!((test_logic, "data/external", "scraper_test"));
