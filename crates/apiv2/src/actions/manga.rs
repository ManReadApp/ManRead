use std::sync::Arc;

use api_structure::{
    models::manga::{
        chapter::Chapter,
        search::{Item, ItemData, ItemOrArray, ItemValue, Order},
        status::Status,
        tag::Tag as GlobalTag,
    },
    req::manga::{
        add::{AddMangaRequest, Scrapers},
        edit::EditMangaRequest,
        search::SearchRequest,
    },
    resp::manga::{home::HomeResponse, info::MangaInfoResponse},
};
use db::{
    auth::RecordData,
    chapter::ChapterDBService,
    error::DbError,
    kind::KindDBService,
    lists::ListDBService,
    manga::{Manga, MangaDBService, Scraper},
    tag::{Tag, TagDBService},
    user::UserDBService,
    version::VersionDBService,
    RecordIdFunc, RecordIdType, DB,
};
use rand::{rng, seq::IteratorRandom as _};
use storage::{CoverFileBuilder, FileBuilderExt as _, FileId, StorageSystem};

use crate::error::ApiError;

struct MangaActions {
    mangas: Arc<MangaDBService>,
    chapters: Arc<ChapterDBService>,
    tags: Arc<TagDBService>,
    kinds: Arc<KindDBService>,
    users: Arc<UserDBService>,
    lists: Arc<ListDBService>,
    versions: Arc<VersionDBService>,
    fs: Arc<StorageSystem>,
}

impl MangaActions {
    async fn prepare_create(
        &self,
        tags: Vec<GlobalTag>,
        scrapers_: Vec<Scrapers>,
        artists: Vec<String>,
        authors: Vec<String>,
        publishers: Vec<String>,
    ) -> ApiResult<(
        Vec<RecordIdType<Tag>>,
        Vec<Scraper>,
        Vec<RecordIdFunc>,
        Vec<RecordIdFunc>,
        Vec<RecordIdFunc>,
    )> {
        let tags = self.tags.get_ids(tags.into_iter()).await?;
        let mut scrapers = vec![];
        for scraper in scrapers_ {
            scrapers.push(Scraper {
                target: self.versions.get(&scraper.channel).await?,
                enabled: true,
                url: scraper.url,
            });
        }
        macro_rules! add_artists {
            ($data:expr) => {{
                let mut add = Vec::new();
                for artist in &$data {
                    add.push(self.users.get_or_create(artist).await?);
                }
                add
            }};
        }

        let artists = add_artists!(artists);
        let authors = add_artists!(authors);
        let publishers = add_artists!(publishers);
        Ok((tags, scrapers, artists, authors, publishers))
    }
}

pub async fn convert_to_search_response(
    v: RecordData<Manga>,
    tag_service: &Arc<TagDBService>,
    rng: &mut ThreadRng,
) -> ApiResult<SearchResponse> {
    let (number, ext) = v
        .data
        .covers
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .choose(rng)
        .unwrap();
    let tags = tag_service
        .get_tags(v.data.tags.into_iter().map(|v| v.thing.id().to_string()))
        .await?;
    Ok(SearchResponse {
        manga_id: v.id.id().to_string(),
        titles: v.data.titles,
        tags,
        status: Status::try_from(v.data.status).unwrap(),
        ext,
        number: number as u32,
    })
}

impl MangaActions {
    pub async fn home(&self, uid: &str) {
        let generate = |order: Order, desc, query| {
            let items = match query {
                None => vec![],
                Some(v) => vec![v],
            };
            SearchRequest {
                order: order.to_string(),
                desc,
                limit: 20,
                page: 1,
                query: Array {
                    not: false,
                    or_post: None,
                    or: false,
                    items,
                },
            }
        };
        let mut rng = rng();

        let search = |req| async {
            let (_, v) = self
                .mangas
                .search(req, RecordIdType::from((User::name(), uid)), false)
                .await?;

            let mut resp: Vec<SearchResponse> = Vec::with_capacity(v.len());
            for v in v {
                resp.push(convert_to_search_response(v, &self.tags, &mut rng).await?);
            }
            Ok::<_, ApiError>(resp)
        };
        let trending = generate(Order::Popularity, true, None);
        let newest = generate(Order::Created, true, None);
        let reading = generate(Order::LastRead, true, None);
        let favorites = generate(
            Order::Alphabetical,
            false,
            Some(ItemOrArray::Item(Item {
                not: false,
                or_post: None,
                data: ItemData {
                    name: "list".to_owned(),
                    value: ItemValue::String("favorites".to_owned()),
                },
            })),
        );
        let latest_updates = generate(Order::Updated, true, None);
        let random = generate(Order::Random, false, None);

        Ok(HomeResponse {
            trending: vec![], //todo: search(trending).await?,
            newest: search(newest).await?,
            latest_updates: search(latest_updates).await?,
            favorites: search(favorites).await?,
            reading: search(reading).await?,
            random: search(random).await?,
        })
    }

    pub async fn search(&self, data: SearchRequest, uid: &str) {
        let (max, search) = self
            .mangas
            .search(data, RecordIdType::from((User::name(), uid)), true)
            .await?;
        let mut rng = rng();

        let mut resp: Vec<SearchResponse> = Vec::with_capacity(search.len());
        for v in search {
            resp.push(convert_to_search_response(v, &self.tags, &mut rng).await?);
        }
    }

    pub async fn info(&self, id: String, uid: &str) {
        let manga = self.mangas.get(&id).await?;
        let chapters_ = self.chapters.get_simple(manga.chapters.into_iter()).await?;
        let mut chapters = vec![];
        for v in chapters_ {
            chapters.push(Chapter {
                id: v.id.id().to_string(),
                titles: v.data.titles,
                chapter: v.data.chapter,
                tags: self
                    .tags
                    .get_tags(v.data.tags.into_iter().map(|v| v.thing.id().to_string()))
                    .await?,
                sources: v.data.sources,
                release_date: v.data.release_date.map(|v| v.to_string()),
            });
        }
        chapters.sort_by(|a, b| a.chapter.partial_cmp(&b.chapter).unwrap_or(Ordering::Equal));
        let resp = MangaInfoResponse {
            titles: manga.titles,
            kind: self.kinds.get_name(manga.kind.clone()).await?,
            description: manga.description,
            tags: self
                .tags
                .get_tags(manga.tags.into_iter().map(|v| v.thing.id().to_string()))
                .await?,
            status: Status::try_from(manga.status).unwrap(),
            visibility: Visibility::try_from(manga.visibility).unwrap(),
            uploader: self
                .users
                .get_name_by_id(manga.uploader.clone())
                .await?
                .data,
            my: manga
                .artists
                .iter()
                .find(|v| v.thing.id().to_string() == uid)
                .is_some()
                || manga
                    .authors
                    .iter()
                    .find(|v| v.thing.id().to_string() == uid)
                    .is_some()
                || manga
                    .publishers
                    .iter()
                    .find(|v| v.thing.id().to_string() == uid)
                    .is_some()
                || manga.uploader.thing.id().to_string() == uid,
            artists: self
                .users
                .get_name_from_ids(manga.artists.into_iter())
                .await?,
            authors: self
                .users
                .get_name_from_ids(manga.authors.into_iter())
                .await?,
            publishers: self
                .users
                .get_name_from_ids(manga.publishers.into_iter())
                .await?,
            cover_ext: manga.covers,
            sources: manga
                .sources
                .into_iter()
                .map(|v| ExternalSite {
                    url: v,
                    //TODO: get icon uri
                    icon_uri: "".to_owned(),
                })
                .collect(),
            relations: self
                .mangas
                .get_names(manga.relations.into_iter(), vec!["en".to_owned()])
                .await?,
            scraper: manga.scraper.iter().any(|v| v.enabled),
            scrapers: {
                let mut scrapers = Vec::new();

                for v in manga.scraper {
                    let target = v
                        .target
                        .get(&*DB)
                        .await?
                        .ok_or(ApiError::DbError(DbError::NotFound))?;

                    scrapers.push(Scrapers {
                        channel: target.data.name,
                        url: v.url,
                    });
                }
                scrapers
            },
            favorite: self.lists.is_favorite(&id, uid).await,
            progress: self.lists.is_reading(&id, uid).await,
            chapters,
            manga_id: id,
        };
    }

    pub async fn delete(&self, id: Strig) {
        self.mangas
            .set_visibility(id, Visibility::AdminReview)
            .await?;
    }

    pub async fn edit(&self, data: EditMangaRequest) {
        let (tags, scrapers, artists, authors, publishers) = self
            .prepare_create(
                data.tags,
                data.scrapers,
                data.artists,
                data.authors,
                data.publishers,
            )
            .await?;
        let kind = self.kinds.get_or_create(&data.kind).await?;
        self.mangas
            .update(
                &data.manga_id,
                data.names,
                data.status,
                data.description,
                tags,
                authors,
                artists,
                publishers,
                data.sources,
                scrapers,
                kind,
            )
            .await?;
        self.mangas.regenerate_tags(&data.manga_id).await?;
    }

    pub async fn create(&self, data: AddMangaRequest, uid: &str) {
        let file = CoverFileBuilder::from(self.fs.take(FileId(data.image_temp_name)).await?);
        let (tags, scrapers, artists, authors, publishers) = self
            .prepare_create(
                data.tags,
                data.scrapers,
                data.artists,
                data.authors,
                data.publishers,
            )
            .await?;
        let manga = Manga {
            titles: data.names,
            kind: self.kinds.get_or_create(&data.kind).await?,
            description: data.description,
            generated_tags: tags.clone(),
            tags,
            status: data.status as u64,
            visibility: Visibility::Visible as u64,
            uploader: RecordIdType::from(RecordId::from((User::name(), &uid))),
            artists: artists
                .into_iter()
                .map(RecordIdType::from)
                .collect::<Vec<_>>(),
            authors: authors
                .into_iter()
                .map(RecordIdType::from)
                .collect::<Vec<_>>(),
            covers: vec![Some(file.ext()?.to_owned())],
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
        let mid = self.mangas.add(manga).await?;
        file.build(&mid.thing.id().to_string()).await?;
    }

    pub async fn add_cover(&self, file_id: &str) {
        todo!()
    }
    pub async fn remove_cover(&self, cover_index: usize) {
        todo!()
    }

    pub async fn add_volume_range(&self, start: f64, end: Option<f64>, title: Option<String>) {}
    pub async fn edit_volume_range(
        &self,
        index: usize,
        start: f64,
        end: Option<f64>,
        title: Option<String>,
    ) {
    }
    pub async fn remove_volume_range(&self, index: usize) {}
}
