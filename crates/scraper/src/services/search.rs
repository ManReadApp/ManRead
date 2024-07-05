use crate::extractor::SearchServiceScrapeData;
use crate::pages::{anilist, animeplanet, kitsu};
use crate::ScrapeError;
use api_structure::models::manga::external_search::{
    ExternalSearchData, ValidSearch, ValidSearches,
};
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use reqwest::Client;
use std::collections::HashMap;
use crate::pages::hidden::multi;

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
    ) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
        if let Some(service) = self.services.get(uri) {
            let (query, page) = search.get_query();
            service.search(&self.client, query, page).await
        } else {
            match uri {
                "anilist" => anilist::search(&self.client, &search.get_simple()?).await,
                "kitsu" => kitsu::search(&self.client, search.get_simple()?).await,
                "anime-planet" => animeplanet::search(&self.client, search.get_simple()?).await,
                _ => multi::manual_search(uri, &self.client, search).await,
            }
        }
    }
}
