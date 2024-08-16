use crate::pages::mangaupdates::search::{
    get_client, Array, Limit, Order, OrderKind, SearchRequest,
};
use crate::ScrapeError;
use api_structure::models::manga::external_search::ExternalSearchData;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;

pub mod data;
pub mod search;

pub async fn search(sd: ExternalSearchData) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
    let client = get_client().await;
    search::search(
        client,
        match sd {
            ExternalSearchData::Advanced(v) => Ok(SearchRequest {
                data: Array::try_from(v.query)?,
                limit: Limit {
                    size: v.limit as u64,
                    page: v.page as u64,
                },
                order: Order {
                    desc: v.desc,
                    kind: OrderKind::try_from(v.order)?,
                },
            }),
            _ => Err(ScrapeError::input_error(
                "only allows advanced search request",
            )),
        }?,
    )
    .await
}
