use std::collections::HashMap;

use actix_web::web::Data;
use actix_web_grants::AuthorityGuard;
use api_structure::models::{
    auth::role::Permission,
    manga::{
        external_search::ValidSearches,
        search::{Field, ItemKind, Order},
    },
};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use manga_scraper::init::Services;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ApiComponent, JsonSchema, Serialize)]
struct StatisitcsResponse {
    info: HashMap<String, Info>,
}
#[derive(Deserialize, ApiComponent, JsonSchema, Serialize)]
struct Info {
    icon_source: Option<String>,
    search: Option<ValidSearches>,
    metadata: bool,
    scraper: bool,
}

#[api_operation(
    tag = "external",
    summary = "Shows which external sites are registered with which capabilities",
    description = r###""###
)]
pub(crate) async fn exec(services: Data<Services>) -> CreatedJson<StatisitcsResponse> {
    let mut res = HashMap::new();
    for service in services.services.iter() {
        let mut info = Info {
            icon_source: service.register.icon_source(),
            search: None,
            metadata: service.metadata.is_some(),
            scraper: false,
        };
        if let Some(v) = &service.searchers {
            info.search = Some(v.query());
        }
        res.insert(service.uri.clone(), info);
    }
    res.insert(
        "Internal".to_string(),
        Info {
            icon_source: None,
            search: Some(ValidSearches::Advanced {
                order_by: vec![
                    Order::Alphabetical,
                    Order::Created,
                    Order::Updated,
                    Order::LastRead,
                    Order::Popularity,
                    Order::Status,
                    Order::ChapterCount,
                ]
                .into_iter()
                .map(|order| order.to_string())
                .collect(),
                fields: vec![
                    Field {
                        name: "title".to_owned(),
                        abbr: vec!["".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "description".to_owned(),
                        abbr: vec![],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "kind".to_owned(),
                        abbr: vec!["k".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "male".to_owned(),
                        abbr: vec!["m".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "female".to_owned(),
                        abbr: vec!["f".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "both".to_owned(),
                        abbr: vec!["b".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "male2female".to_owned(),
                        abbr: vec!["mf".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "female2male".to_owned(),
                        abbr: vec!["fm".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "status".to_owned(),
                        abbr: vec!["s".to_owned()],
                        kind: ItemKind::Int,
                    },
                    Field {
                        name: "tag".to_owned(),
                        abbr: vec!["t".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "unknown".to_owned(),
                        abbr: vec!["u".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "none".to_owned(),
                        abbr: vec!["n".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "uploader".to_owned(),
                        abbr: vec![],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "artist".to_owned(),
                        abbr: vec![],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "author".to_owned(),
                        abbr: vec!["a".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "publisher".to_owned(),
                        abbr: vec!["p".to_owned()],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "chapters".to_owned(),
                        abbr: vec!["c".to_owned()],
                        kind: ItemKind::CmpFloat,
                    },
                    Field {
                        name: "list".to_owned(),
                        abbr: vec![],
                        kind: ItemKind::String,
                    },
                    Field {
                        name: "next-available".to_owned(),
                        abbr: vec![],
                        kind: ItemKind::None,
                    },
                ],
            }),
            metadata: false,
            scraper: false,
        },
    );
    CreatedJson(StatisitcsResponse { info: res })
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/statistics").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
