use crate::extractor::SearchServiceScrapeData;
use crate::pages::{anilist, animeplanet, kitsu};
use crate::ScrapeError;
use api_structure::scraper::{ExternalSearchData, ScrapeSearchResult, ValidSearch, ValidSearches};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchService {
    client: Client,
    services: HashMap<String, SearchServiceScrapeData>,
}

impl SearchService {
    pub fn new(services: HashMap<String, SearchServiceScrapeData>) -> Self {
        Self {
            client: Default::default(),
            services,
        }
    }

    pub fn sites(&self) -> HashMap<String, ValidSearches> {
        let mut keys = vec![
            (
                "kitsu".to_string(),
                ValidSearches::ValidSearch(ValidSearch::kitsu()),
            ),
            (
                "anilist".to_string(),
                ValidSearches::ValidSearch(ValidSearch::anilist()),
            ),
            (
                "anime-planet".to_string(),
                ValidSearches::ValidSearch(animeplanet::get_valid()),
            ),
        ];
        keys.append(
            &mut self
                .services
                .keys()
                .cloned()
                .map(|v| (v, ValidSearches::String))
                .collect::<Vec<_>>(),
        );
        keys.into_iter().collect()
    }

    pub async fn search(
        &self,
        uri: &str,
        search: ExternalSearchData,
    ) -> Result<Vec<ScrapeSearchResult>, ScrapeError> {
        if let Some(service) = self.services.get(uri) {
            let (query, page) = search.get_query();
            service.search(&self.client, query, page).await
        } else {
            match uri {
                "anilist" => anilist::search(&self.client, &search.get_simple()?).await,
                "kitsu" => kitsu::search(&self.client, search.get_simple()?).await,
                "anime-planet" => animeplanet::search(&self.client, search.get_simple()?).await,
                _ => Err(ScrapeError::input_error("uri does not exist")),
            }
        }
    }
}
