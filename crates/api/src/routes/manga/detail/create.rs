use crate::{
    error::ApiResult,
    models::{
        kind::KindDBService,
        manga::{Manga, MangaDBService, Scraper},
        tag::{Tag, TagDBService},
        user::{User, UserDBService},
        version::VersionDBService,
    },
    services::file::FileService,
    Config,
};
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{jwt::Claim, role::Permission},
        manga::{tag::Tag as GlobalTag, visiblity::Visibility},
    },
    req::manga::add::{AddMangaRequest, Scrapers},
};
use apistos::{actix::CreatedJson, api_operation};
use surrealdb::RecordId;
use surrealdb_extras::{RecordIdFunc, RecordIdType, SurrealTableInfo};

pub async fn prepare(
    tags: Vec<GlobalTag>,
    scrapers_: Vec<Scrapers>,
    artists: Vec<String>,
    authors: Vec<String>,
    publishers: Vec<String>,
    tag_service: Data<TagDBService>,
    version_service: Data<VersionDBService>,
    user_service: Data<UserDBService>,
) -> ApiResult<(
    Vec<RecordIdType<Tag>>,
    Vec<Scraper>,
    Vec<RecordIdFunc>,
    Vec<RecordIdFunc>,
    Vec<RecordIdFunc>,
)> {
    let tags = tag_service.get_ids(tags.into_iter()).await?;
    let mut scrapers = vec![];
    for scraper in scrapers_ {
        scrapers.push(Scraper {
            target: version_service.get(&scraper.channel).await?,
            enabled: true,
            url: scraper.url,
        });
    }
    macro_rules! add_artists {
        ($data:expr) => {{
            let mut add = Vec::new();
            for artist in &$data {
                add.push(user_service.get_or_create(artist).await?);
            }
            add
        }};
    }

    let artists = add_artists!(artists);
    let authors = add_artists!(authors);
    let publishers = add_artists!(publishers);
    Ok((tags, scrapers, artists, authors, publishers))
}
#[api_operation(
    tag = "manga",
    summary = "Creates a manga",
    description = r###"Returns the manga id"###
)]
pub(crate) async fn exec(
    Json(data): Json<AddMangaRequest>,
    manga_service: Data<MangaDBService>,
    kind_service: Data<KindDBService>,
    tag_service: Data<TagDBService>,
    file_service: Data<FileService>,
    version_service: Data<VersionDBService>,
    user_service: Data<UserDBService>,
    uploader: ReqData<Claim>,
    config: Data<Config>,
) -> ApiResult<CreatedJson<String>> {
    let file = file_service.take(&data.image_temp_name)?;
    let (tags, scrapers, artists, authors, publishers) = prepare(
        data.tags,
        data.scrapers,
        data.artists,
        data.authors,
        data.publishers,
        tag_service,
        version_service,
        user_service,
    )
    .await?;
    let manga = Manga {
        titles: data.names,
        kind: kind_service.get_or_create(&data.kind).await?,
        description: data.description,
        generated_tags: tags.clone(),
        tags,
        status: data.status as u64,
        visibility: Visibility::Visible as u64,
        uploader: RecordIdType::from(RecordId::from((User::name(), &uploader.id))),
        artists: artists
            .into_iter()
            .map(RecordIdType::from)
            .collect::<Vec<_>>(),
        authors: authors
            .into_iter()
            .map(RecordIdType::from)
            .collect::<Vec<_>>(),
        covers: vec![Some(file.ext().to_owned())],
        chapters: vec![],
        sources: data.sources,
        relations: vec![],
        scraper: scrapers,
        updated: Default::default(),
        created: Default::default(),
        art_ext: vec![],
        publishers: publishers
            .into_iter()
            .map(RecordIdType::from)
            .collect::<Vec<_>>(),
        volumes: vec![],
    };
    let mid = manga_service.add(manga).await?;
    file.move_to(&config.root_folder, "covers", &mid.thing.id().to_string());
    Ok(CreatedJson(mid.thing.id().to_string()))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/create").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
