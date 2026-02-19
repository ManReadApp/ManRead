use crate::error::{ApiError, ApiResult};
use api_structure::{
    search::{
        Array, HomeResponse, Item, ItemData, ItemOrArray, ItemValue, Order, SearchRequest,
        SearchResponse,
    },
    v1::{
        self, AddMangaRequest, Chapter, EditMangaRequest, ExternalSite, MangaInfoResponse,
        Relation, Status, Tag as GlobalTag, Visibility,
    },
};
use bytes::Bytes;
use db::{
    auth::RecordData,
    chapter::ChapterDBService,
    kind::KindDBService,
    lists::ListDBService,
    manga::{Manga, MangaDBService, Scraper},
    page::PageDBService,
    tag::{Tag, TagDBService},
    user::{User, UserDBService},
    version::VersionDBService,
    version_link::ChapterVersionDBService,
    RecordId, RecordIdFunc, RecordIdType, SurrealTableInfo,
};
use futures_util::{stream, Stream, StreamExt as _};
use rand::{rng, rngs::ThreadRng, seq::IteratorRandom as _};
use std::{cmp::Ordering, pin::Pin, sync::Arc, task::Poll};
use storage::{ArtFileBuilder, CoverFileBuilder, FileBuilderExt as _, FileId, StorageSystem};

pub struct MangaActions {
    pub mangas: Arc<MangaDBService>,
    pub chapters: Arc<ChapterDBService>,
    pub tags: Arc<TagDBService>,
    pub kinds: Arc<KindDBService>,
    pub users: Arc<UserDBService>,
    pub lists: Arc<ListDBService>,
    pub versions: Arc<VersionDBService>,
    pub chapter_versions: Arc<ChapterVersionDBService>,
    pub pages: Arc<PageDBService>,
    pub fs: Arc<StorageSystem>,
}

fn validate_non_empty(field: &str, value: &str) -> ApiResult<()> {
    if value.trim().is_empty() {
        return Err(ApiError::invalid_input(&format!("{field} cannot be empty")));
    }
    Ok(())
}

fn validate_non_empty_items(field: &str, items: &[String]) -> ApiResult<()> {
    if items.iter().any(|v| v.trim().is_empty()) {
        return Err(ApiError::invalid_input(&format!(
            "{field} cannot contain empty values"
        )));
    }
    Ok(())
}

fn validate_titles(names: &std::collections::HashMap<String, v1::StringList>) -> ApiResult<()> {
    if names.is_empty() {
        return Err(ApiError::invalid_input("names cannot be empty"));
    }
    for (lang, titles) in names {
        validate_non_empty("language", lang)?;
        if titles.items.is_empty() {
            return Err(ApiError::invalid_input("name list cannot be empty"));
        }
        validate_non_empty_items("names", &titles.items)?;
    }
    Ok(())
}

fn validate_scrapers(scrapers: &[v1::Scraper]) -> ApiResult<()> {
    for scraper in scrapers {
        validate_non_empty("scraper.channel", &scraper.channel)?;
        validate_non_empty("scraper.url", &scraper.url)?;
    }
    Ok(())
}

fn validate_pagination(page: u32, limit: u32) -> ApiResult<()> {
    if page == 0 {
        return Err(ApiError::invalid_input("page must be >= 1"));
    }
    if limit == 0 {
        return Err(ApiError::invalid_input("limit must be >= 1"));
    }
    Ok(())
}

fn status_from_db(value: u64) -> ApiResult<Status> {
    Status::try_from(value).map_err(|_| ApiError::write_error("invalid status value in database"))
}

fn visibility_from_db(value: u64) -> ApiResult<Visibility> {
    Visibility::try_from(value)
        .map_err(|_| ApiError::write_error("invalid visibility value in database"))
}

const MANGA_CONTAINER_MAGIC: &[u8; 8] = b"MRMANG01";

pub type ExportStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

enum ExportSegment {
    Bytes(Bytes),
    Object { key: String, len: u64 },
}

impl ExportSegment {
    fn len(&self) -> u64 {
        match self {
            Self::Bytes(v) => v.len() as u64,
            Self::Object { len, .. } => *len,
        }
    }
}

pub struct PreparedExport {
    segments: Vec<ExportSegment>,
    total_len: u64,
}

pub struct ExportOutput {
    pub stream: ExportStream,
    pub range: (u64, u64),
}

fn len_to_u32(len: usize, what: &str) -> ApiResult<u32> {
    u32::try_from(len).map_err(|_| ApiError::invalid_input(&format!("{what} exceeds limit")))
}

fn chunk_stream(chunk: Bytes) -> ExportStream {
    Box::pin(stream::once(async move { Ok(chunk) }))
}

fn slice_stream(mut source: ExportStream, mut skip: u64, mut remaining: u64) -> ExportStream {
    Box::pin(stream::poll_fn(move |cx| loop {
        if remaining == 0 {
            return Poll::Ready(None);
        }

        match source.as_mut().poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
            Poll::Ready(Some(Ok(chunk))) => {
                let chunk_len = chunk.len() as u64;
                if skip >= chunk_len {
                    skip -= chunk_len;
                    continue;
                }

                let mut out = chunk;
                if skip > 0 {
                    out = out.slice(skip as usize..);
                    skip = 0;
                }

                if remaining < out.len() as u64 {
                    let take = remaining as usize;
                    remaining = 0;
                    return Poll::Ready(Some(Ok(out.slice(..take))));
                }

                remaining -= out.len() as u64;
                return Poll::Ready(Some(Ok(out)));
            }
        }
    }))
}

impl MangaActions {
    async fn prepare_create(
        &self,
        tags: Vec<GlobalTag>,
        scrapers_: Vec<v1::Scraper>,
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
        .ok_or(ApiError::invalid_input("No cover available"))?;
    let tags = tag_service
        .get_tags(v.data.tags.into_iter().map(|v| v.thing.id().to_string()))
        .await?;
    Ok(SearchResponse {
        manga_id: v.id.id().to_string(),
        titles: v.data.titles,
        tags,
        status: status_from_db(v.data.status)?,
        ext,
        number: number as u32,
    })
}

impl MangaActions {
    pub async fn home(&self, uid: &str) -> ApiResult<HomeResponse> {
        validate_non_empty("uid", uid)?;
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

        let search = |req| async {
            let mut rng = rng();
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
        let _trending = generate(Order::Popularity, true, None);
        let newest = generate(Order::Created, true, None);
        let reading = generate(
            Order::LastRead,
            true,
            Some(ItemOrArray::Item(Item::new(ItemData::enum_(
                "next-available",
            )))),
        );
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

    pub async fn search(
        &self,
        data: SearchRequest,
        uid: &str,
    ) -> ApiResult<(Vec<SearchResponse>, u64)> {
        validate_non_empty("uid", uid)?;
        validate_pagination(data.page, data.limit)?;
        Order::try_from(data.order.clone())
            .map_err(|v| ApiError::invalid_input(v.message().as_str()))?;
        let (max, search) = self
            .mangas
            .search(data, RecordIdType::from((User::name(), uid)), true)
            .await?;
        let mut rng = rng();

        let mut resp: Vec<SearchResponse> = Vec::with_capacity(search.len());
        for v in search {
            resp.push(convert_to_search_response(v, &self.tags, &mut rng).await?);
        }
        Ok((resp, max))
    }

    pub async fn info(&self, id: String, uid: &str) -> ApiResult<MangaInfoResponse> {
        validate_non_empty("manga_id", &id)?;
        validate_non_empty("uid", uid)?;
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
        Ok(MangaInfoResponse {
            titles: manga
                .titles
                .into_iter()
                .map(|v| (v.0, v.1.into()))
                .collect(),
            kind: self.kinds.get_name(manga.kind.clone()).await?,
            description: manga.description,
            tags: self
                .tags
                .get_tags(manga.tags.into_iter().map(|v| v.thing.id().to_string()))
                .await?,
            status: status_from_db(manga.status)?,
            visibility: visibility_from_db(manga.visibility)?,
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
            cover_ext: manga.covers.into_iter().map(|v| v.into()).collect(),
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
                .await?
                .into_iter()
                .map(|v| Relation {
                    manga_id: v.0,
                    kind: v.1,
                })
                .collect(),
            scraper: manga.scraper.iter().any(|v| v.enabled),
            scrapers: {
                let mut scrapers = Vec::new();

                for v in manga.scraper {
                    let target = self.versions.get_(v.target).await?;

                    scrapers.push(api_structure::v1::Scraper {
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
        })
    }

    pub async fn prepare_export(&self, manga_id: &str) -> ApiResult<PreparedExport> {
        validate_non_empty("manga_id", manga_id)?;
        let manga = self.mangas.get(manga_id).await?;
        let mut chapters = self.chapters.get_detail(manga.chapters.into_iter()).await?;
        chapters.sort_by(|a, b| {
            a.data
                .chapter
                .partial_cmp(&b.data.chapter)
                .unwrap_or(Ordering::Equal)
        });

        let mut metadata_chapters = Vec::with_capacity(chapters.len());
        let mut image_keys = Vec::new();

        for chapter in chapters {
            let chapter_id = chapter.id.id().to_string();
            let mut versions: Vec<_> = chapter.data.versions.into_iter().collect();
            versions.sort_by(|a, b| a.0.cmp(&b.0));

            let Some((version_key, chapter_version)) = versions.into_iter().next() else {
                return Err(ApiError::invalid_input(&format!(
                    "chapter {chapter_id} has no versions"
                )));
            };

            let version_id = version_key
                .split_once(':')
                .map(|(_, id)| id.to_owned())
                .unwrap_or(version_key);

            let chapter_version = self
                .chapter_versions
                .get(&chapter_version.id().to_string())
                .await?;
            let mut pages = self.pages.get(chapter_version.pages).await?;
            pages.sort_by_key(|page| page.data.page);

            let mut image_indexes = Vec::with_capacity(pages.len());
            for page in pages {
                image_indexes.push(len_to_u32(image_keys.len(), "image index")?);
                let ext = page.data.ext.trim_start_matches('.');
                let key = format!(
                    "mangas/{}/{}/{}/{}.{}",
                    manga_id, chapter_id, version_id, page.data.page, ext
                );
                image_keys.push(key);
            }

            metadata_chapters.push(export::manga::Chapter {
                chapter_id,
                version_id,
                image_indexes,
            });
        }

        let metadata = export::manga::MangaBundleMetadata {
            manga_id: manga_id.to_owned(),
            chapters: metadata_chapters,
        };
        let metadata_bytes = export::to_bytes(&metadata);
        let metadata_len = len_to_u32(metadata_bytes.len(), "metadata size")?.to_le_bytes();
        let image_count = len_to_u32(image_keys.len(), "image count")?.to_le_bytes();

        let mut segments: Vec<ExportSegment> = Vec::with_capacity(4 + image_keys.len() * 2);
        segments.push(ExportSegment::Bytes(Bytes::from_static(
            MANGA_CONTAINER_MAGIC,
        )));
        segments.push(ExportSegment::Bytes(Bytes::copy_from_slice(&metadata_len)));
        segments.push(ExportSegment::Bytes(metadata_bytes));
        segments.push(ExportSegment::Bytes(Bytes::copy_from_slice(&image_count)));

        for key in image_keys {
            let object = self.fs.reader.get(&key, &Default::default()).await?;
            let image_len = object.content_length.ok_or_else(|| {
                ApiError::write_error(format!("missing content length for {key}"))
            })?;
            let image_len_u32 = u32::try_from(image_len).map_err(|_| {
                ApiError::invalid_input(&format!("image size exceeds limit for {key}"))
            })?;

            segments.push(ExportSegment::Bytes(Bytes::copy_from_slice(
                &image_len_u32.to_le_bytes(),
            )));
            segments.push(ExportSegment::Object {
                key,
                len: image_len,
            });
        }

        let total_len = segments
            .iter()
            .try_fold(0u64, |acc, seg| acc.checked_add(seg.len()))
            .ok_or_else(|| ApiError::write_error("export payload too large"))?;

        Ok(PreparedExport {
            segments,
            total_len,
        })
    }

    pub async fn export(&self, manga_id: &str) -> ApiResult<ExportStream> {
        let prepared = self.prepare_export(manga_id).await?;
        let output = prepared.into_stream(&self.fs, None).await?;
        Ok(output.stream)
    }
}

impl PreparedExport {
    pub fn total_len(&self) -> u64 {
        self.total_len
    }

    pub async fn into_stream(
        self,
        fs: &StorageSystem,
        range: Option<(u64, u64)>,
    ) -> ApiResult<ExportOutput> {
        let total = self.total_len;
        if total == 0 {
            return Err(ApiError::write_error("empty export payload"));
        }
        let (start, end) = range.unwrap_or((0, total - 1));
        if start > end || end >= total {
            return Err(ApiError::invalid_input("invalid byte range"));
        }

        let mut cursor = 0u64;
        let mut streams: Vec<ExportStream> = Vec::new();

        for seg in self.segments {
            let seg_len = seg.len();
            let seg_start = cursor;
            let seg_end = cursor + seg_len - 1;
            cursor += seg_len;

            if end < seg_start || start > seg_end {
                continue;
            }

            let local_start = start.saturating_sub(seg_start);
            let local_end = if end < seg_end {
                end - seg_start
            } else {
                seg_len - 1
            };
            let take = local_end - local_start + 1;

            match seg {
                ExportSegment::Bytes(bytes) => {
                    let s = local_start as usize;
                    let e = (local_start + take) as usize;
                    streams.push(chunk_stream(bytes.slice(s..e)));
                }
                ExportSegment::Object { key, .. } => {
                    let source = fs.reader.get(&key, &Default::default()).await?.stream;
                    streams.push(slice_stream(source, local_start, take));
                }
            }
        }

        Ok(ExportOutput {
            stream: Box::pin(stream::iter(streams).flatten()),
            range: (start, end),
        })
    }
}

impl MangaActions {
    pub async fn delete(&self, id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", id)?;
        self.mangas
            .set_visibility(id, Visibility::AdminReview)
            .await?;
        Ok(())
    }

    pub async fn edit(&self, data: EditMangaRequest) -> ApiResult<()> {
        validate_non_empty("manga_id", &data.manga_id)?;
        validate_non_empty("kind", &data.kind)?;
        validate_titles(&data.names)?;
        validate_non_empty_items("authors", &data.authors)?;
        validate_non_empty_items("artists", &data.artists)?;
        validate_non_empty_items("publishers", &data.publishers)?;
        validate_non_empty_items("sources", &data.sources)?;
        validate_scrapers(&data.scrapers)?;
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
                data.names.into_iter().map(|v| (v.0, v.1.into())).collect(),
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
        Ok(())
    }

    pub async fn create(&self, data: AddMangaRequest, uid: &str) -> ApiResult<()> {
        validate_non_empty("uid", uid)?;
        validate_non_empty("kind", &data.kind)?;
        validate_non_empty("image_temp_name", &data.image_temp_name)?;
        validate_titles(&data.names)?;
        validate_non_empty_items("authors", &data.authors)?;
        validate_non_empty_items("artists", &data.artists)?;
        validate_non_empty_items("publishers", &data.publishers)?;
        validate_non_empty_items("sources", &data.sources)?;
        validate_scrapers(&data.scrapers)?;
        let file = CoverFileBuilder::from(self.fs.take(FileId::new(data.image_temp_name)).await?);
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
            titles: data.names.into_iter().map(|v| (v.0, v.1.into())).collect(),
            kind: self.kinds.get_or_create(&data.kind).await?,
            description: data.description,
            generated_tags: tags.clone(),
            tags,
            status: data.status as u64,
            visibility: Visibility::Visible as u64,
            uploader: RecordIdType::from(RecordId::from((User::name(), uid))),
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
        file.build(&mid.thing.id().to_string(), 0).await?;
        Ok(())
    }

    pub async fn add_cover(&self, mid: &str, file_id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("file_id", file_id)?;
        let file = CoverFileBuilder::from(self.fs.take(FileId::new(file_id.to_owned())).await?);
        let index = self.mangas.add_cover(mid, file.ext()?).await?;
        file.build(mid, index).await?;

        Ok(())
    }

    pub async fn remove_cover(&self, mid: &str, cover_index: usize) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        let manga = self.mangas.get(mid).await?;
        if cover_index >= manga.covers.len() {
            return Err(ApiError::invalid_input("cover_index out of bounds"));
        }
        if manga
            .covers
            .get(cover_index)
            .and_then(|value| value.as_ref())
            .is_none()
        {
            return Err(ApiError::invalid_input("cover does not exist"));
        }
        if manga
            .covers
            .iter()
            .enumerate()
            .filter(|(idx, value)| *idx != cover_index && value.is_some())
            .count()
            == 0
        {
            return Err(ApiError::invalid_input("at least one cover is required"));
        }
        self.mangas.remove_cover(mid, cover_index).await?;
        Ok(())
    }

    pub async fn add_art(&self, mid: &str, file_id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("file_id", file_id)?;
        let file = ArtFileBuilder::from(self.fs.take(FileId::new(file_id.to_owned())).await?);
        let index = self.mangas.add_art(mid, file.ext()?).await?;
        file.build(mid, index).await?;

        Ok(())
    }

    pub async fn remove_art(&self, mid: &str, file_index: usize) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        let manga = self.mangas.get(mid).await?;
        if file_index >= manga.art_ext.len() {
            return Err(ApiError::invalid_input("art_index out of bounds"));
        }
        if manga
            .art_ext
            .get(file_index)
            .and_then(|value| value.as_ref())
            .is_none()
        {
            return Err(ApiError::invalid_input("art does not exist"));
        }
        self.mangas.remove_art(mid, file_index).await?;
        Ok(())
    }

    pub async fn confirm_delete(&self, id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", id)?;
        self.mangas.set_visibility(id, Visibility::Hidden).await?;
        Ok(())
    }

    pub async fn set_volume_range(&self, id: &str, vols: Vec<VolumeRange>) -> ApiResult<()> {
        validate_non_empty("manga_id", id)?;
        for vol in &vols {
            if !vol.start.is_finite() || vol.start < 0.0 {
                return Err(ApiError::invalid_input(
                    "volume start must be finite and >= 0",
                ));
            }
            if let Some(end) = vol.end {
                if !end.is_finite() || end < vol.start {
                    return Err(ApiError::invalid_input(
                        "volume end must be finite and >= start",
                    ));
                }
            }
        }
        self.mangas
            .set_volumes(
                id,
                vols.into_iter()
                    .map(|v| (v.title, v.start, v.end))
                    .collect(),
            )
            .await?;
        Ok(())
    }

    pub async fn add_relation(&self, mid: &str, relation_id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("relation_id", relation_id)?;
        if mid == relation_id {
            return Err(ApiError::invalid_input(
                "manga_id and relation_id cannot be the same",
            ));
        }
        self.mangas.exists(mid).await?;
        self.mangas.exists(relation_id).await?;
        self.mangas.add_relation(relation_id, mid).await?;
        self.mangas.add_relation(mid, relation_id).await?;
        Ok(())
    }

    pub async fn remove_relation(&self, mid: &str, relation_id: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("relation_id", relation_id)?;
        self.mangas.remove_relation(mid, relation_id).await?;
        self.mangas.remove_relation(relation_id, mid).await?;
        Ok(())
    }

    pub async fn disable_scraper(&self, mid: &str, url: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("url", url)?;
        self.mangas.set_scraper(mid, url, false).await?;
        Ok(())
    }

    pub async fn enable_scraper(&self, mid: &str, url: &str) -> ApiResult<()> {
        validate_non_empty("manga_id", mid)?;
        validate_non_empty("url", url)?;
        self.mangas.set_scraper(mid, url, true).await?;
        Ok(())
    }
}

pub struct VolumeRange {
    pub start: f64,
    pub end: Option<f64>,
    pub title: Option<String>,
}
