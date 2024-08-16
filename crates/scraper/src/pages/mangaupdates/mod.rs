use crate::pages::mangaupdates::search::get_client;
use crate::ScrapeError;
use api_structure::models::manga::external_search::ExternalSearchData;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;

pub mod data;
pub mod search;

pub async fn search(sd: ExternalSearchData) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
    let client = get_client().await;
    search::search(client, todo!()).await
}
