use crate::extractor::SearchServiceScrapeData;
use crate::pages::hidden::multi;
use crate::pages::{anilist, animeplanet, kitsu, mangadex};
use crate::ScrapeError;
use api_structure::models::manga::external_search::{
    ExternalSearchData, ValidSearch, ValidSearches,
};
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use crate::services::MangaData;

#[derive(Default)]
pub struct SearchService {
    client: Client,
    services: HashMap<String, SearchServiceScrapeData>,
    local_services: Arc<HashMap<String, HashMap<String, MangaData>>>
}

impl SearchService {
    pub fn new(services: HashMap<String, SearchServiceScrapeData>, local_services: Arc<HashMap<String, HashMap<String, MangaData>>>) -> Self {
        Self {
            client: Default::default(),
            services,
            local_services
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
        } else if let Some(service) = self.local_services.get(uri) {
            todo!()
        }else {
            match uri {
                "anilist" => anilist::search(&self.client, &search.get_simple()?).await,
                "kitsu" => kitsu::search(&self.client, search.get_simple()?).await,
                "anime-planet" => animeplanet::search(&self.client, search.get_simple()?).await,
                "mangadex" => mangadex::search(&self.client, search).await,
                _ => multi::manual_search(uri, &self.client, search).await,
            }
        }
    }
}
