use crate::{
    actions::chapter::ChapterActions,
    error::{ApiError, ApiResult},
};
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
use chrono::{DateTime, Utc};
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
use std::{cmp::Ordering, collections::HashMap, pin::Pin, sync::Arc, task::Poll};
use storage::{
    ArtFileBuilder, CoverFileBuilder, FileBuilderExt as _, FileId, RegisterTempResult,
    StorageSystem,
};
use tokio::io::AsyncWriteExt as _;

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

fn export_tag_from_global(tag: GlobalTag) -> export::manga::Tag {
    export::manga::Tag {
        tag: tag.tag,
        description: tag.description,
        sex: tag.sex as u64,
    }
}

fn global_tag_from_export(tag: export::manga::Tag) -> GlobalTag {
    GlobalTag {
        tag: tag.tag,
        description: tag.description,
        sex: api_structure::v1::TagSex::try_from(tag.sex)
            .unwrap_or(api_structure::v1::TagSex::Unknown),
    }
}

fn sanitize_non_empty(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_release_date(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|raw| {
        DateTime::parse_from_rfc3339(&raw)
            .ok()
            .map(|date| date.with_timezone(&Utc))
    })
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
        let kind = self.kinds.get_name(manga.kind.clone()).await?;
        let uploader = self
            .users
            .get_name_by_id(manga.uploader.clone())
            .await?
            .data;
        let artists = self
            .users
            .get_name_from_ids(manga.artists.clone().into_iter())
            .await?;
        let authors = self
            .users
            .get_name_from_ids(manga.authors.clone().into_iter())
            .await?;
        let publishers = self
            .users
            .get_name_from_ids(manga.publishers.clone().into_iter())
            .await?;
        let tags = self
            .tags
            .get_tags(
                manga
                    .tags
                    .clone()
                    .into_iter()
                    .map(|v| v.thing.id().to_string()),
            )
            .await?
            .into_iter()
            .map(export_tag_from_global)
            .collect::<Vec<_>>();
        let mut metadata_scraper = Vec::with_capacity(manga.scraper.len());
        for scraper in manga.scraper.clone() {
            let channel = self.versions.get_(scraper.target).await?.data.name;
            metadata_scraper.push(export::manga::Scraper {
                channel,
                url: scraper.url,
                enabled: scraper.enabled,
            });
        }

        let mut metadata = export::manga::MangaBundleMetadata {
            titles: manga
                .titles
                .clone()
                .into_iter()
                .map(|(lang, titles)| (lang, export::manga::StringList { items: titles }))
                .collect(),
            kind,
            description: manga.description.clone(),
            tags,
            status: manga.status,
            visibility: manga.visibility,
            uploader,
            artists,
            authors,
            publishers,
            sources: manga.sources.clone(),
            scraper: metadata_scraper,
            volumes: manga
                .volumes
                .clone()
                .into_iter()
                .map(|v| export::manga::Volume {
                    title: v.title,
                    start: v.start,
                    end: v.end,
                })
                .collect(),
            cover_image_indexes: vec![],
            art_image_indexes: vec![],
            chapters: vec![],
        };

        let mut image_keys = Vec::new();
        for (index, ext) in manga.covers.iter().enumerate() {
            let Some(ext) = ext.as_ref() else {
                continue;
            };
            metadata
                .cover_image_indexes
                .push(len_to_u32(image_keys.len(), "image index")?);
            let ext = ext.trim_start_matches('.');
            let key = if index == 0 {
                format!("covers/{manga_id}.{ext}")
            } else {
                format!("covers/{}_{}.{}", manga_id, index, ext)
            };
            image_keys.push(key);
        }
        for (index, ext) in manga.art_ext.iter().enumerate() {
            let Some(ext) = ext.as_ref() else {
                continue;
            };
            metadata
                .art_image_indexes
                .push(len_to_u32(image_keys.len(), "image index")?);
            let ext = ext.trim_start_matches('.');
            image_keys.push(format!("arts/{}_{}.{}", manga_id, index, ext));
        }

        let mut chapters = self
            .chapters
            .get_detail(manga.chapters.clone().into_iter())
            .await?;
        chapters.sort_by(|a, b| {
            a.data
                .chapter
                .partial_cmp(&b.data.chapter)
                .unwrap_or(Ordering::Equal)
        });

        for chapter in chapters {
            let chapter_id = chapter.id.id().to_string();
            let chapter_tags = self
                .tags
                .get_tags(
                    chapter
                        .data
                        .tags
                        .clone()
                        .into_iter()
                        .map(|v| v.thing.id().to_string()),
                )
                .await?
                .into_iter()
                .map(export_tag_from_global)
                .collect::<Vec<_>>();
            let mut versions = Vec::new();
            for (_, chapter_version_ref) in chapter.data.versions.clone() {
                let chapter_version = self
                    .chapter_versions
                    .get(&chapter_version_ref.id().to_string())
                    .await?;
                let version_name = self
                    .versions
                    .get_(chapter_version.version.clone())
                    .await?
                    .data
                    .name;
                let version_id = chapter_version.version.id().to_string();
                versions.push((version_name, version_id, chapter_version));
            }
            versions.sort_by(|a, b| a.0.cmp(&b.0));
            if versions.is_empty() {
                return Err(ApiError::invalid_input(&format!(
                    "chapter {chapter_id} has no versions"
                )));
            }

            let mut metadata_versions = Vec::with_capacity(versions.len());
            for (version_name, version_id, chapter_version) in versions {
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
                metadata_versions.push(export::manga::ChapterVersion {
                    version: version_name,
                    image_indexes,
                    link: chapter_version.link,
                });
            }

            metadata.chapters.push(export::manga::Chapter {
                titles: chapter.data.titles,
                chapter: chapter.data.chapter,
                tags: chapter_tags,
                sources: chapter.data.sources,
                release_date: chapter.data.release_date.map(|v| v.to_string()),
                versions: metadata_versions,
            });
        }
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

    async fn upload_bytes_as_file_id(&self, bytes: &[u8]) -> ApiResult<String> {
        let mut temp_file = self.fs.new_temp_file().await?;
        temp_file
            .write_all(bytes)
            .await
            .map_err(ApiError::write_error)?;
        temp_file.flush().await.map_err(ApiError::write_error)?;
        let upload = self.fs.register_temp_file(temp_file).await?;
        match upload {
            RegisterTempResult::File(file_id) => Ok(file_id.inner()),
            RegisterTempResult::Chapter(_) | RegisterTempResult::Manga(_) => Err(
                ApiError::invalid_input("restore expected a single image temp file"),
            ),
        }
    }

    fn build_restore_create_request(
        metadata: &export::manga::MangaBundleMetadata,
        image_temp_name: String,
    ) -> AddMangaRequest {
        let mut names = metadata
            .titles
            .clone()
            .into_iter()
            .filter_map(|(lang, titles)| {
                let items = sanitize_non_empty(titles.items);
                if lang.trim().is_empty() || items.is_empty() {
                    None
                } else {
                    Some((lang, v1::StringList { items }))
                }
            })
            .collect::<HashMap<_, _>>();
        if names.is_empty() {
            names.insert(
                "en".to_owned(),
                v1::StringList {
                    items: vec!["Restored Manga".to_owned()],
                },
            );
        }

        let kind = if metadata.kind.trim().is_empty() {
            "restored".to_owned()
        } else {
            metadata.kind.trim().to_owned()
        };
        let status = Status::try_from(metadata.status).unwrap_or(Status::Ongoing);
        let description = metadata.description.clone().and_then(|value| {
            let value = value.trim().to_owned();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        });

        AddMangaRequest {
            names,
            kind,
            status,
            description,
            tags: metadata
                .tags
                .clone()
                .into_iter()
                .map(global_tag_from_export)
                .collect(),
            image_temp_name,
            authors: sanitize_non_empty(metadata.authors.clone()),
            publishers: sanitize_non_empty(metadata.publishers.clone()),
            artists: sanitize_non_empty(metadata.artists.clone()),
            sources: sanitize_non_empty(metadata.sources.clone()),
            scrapers: metadata
                .scraper
                .clone()
                .into_iter()
                .filter(|scraper| scraper.enabled)
                .filter_map(|scraper| {
                    let channel = scraper.channel.trim().to_owned();
                    let url = scraper.url.trim().to_owned();
                    if channel.is_empty() || url.is_empty() {
                        None
                    } else {
                        Some(v1::Scraper { channel, url })
                    }
                })
                .collect(),
        }
    }

    pub async fn restore(
        &self,
        metadata_id: &str,
        image_ids: Vec<String>,
        uid: &str,
    ) -> ApiResult<()> {
        validate_non_empty("metadata_id", metadata_id)?;
        validate_non_empty("uid", uid)?;
        if image_ids.iter().any(|id| id.trim().is_empty()) {
            return Err(ApiError::invalid_input(
                "image_ids cannot contain empty values",
            ));
        }

        let metadata_bytes = self
            .fs
            .take_bytes(FileId::new(metadata_id.to_owned()))
            .await?;
        let metadata: export::manga::MangaBundleMetadata = export::try_from_bytes(&metadata_bytes)
            .map_err(|e| ApiError::invalid_input(&format!("invalid export metadata: {e}")))?;

        let register_ref = |index: u32, counts: &mut Vec<usize>| -> ApiResult<()> {
            let idx = usize::try_from(index)
                .map_err(|_| ApiError::invalid_input("invalid image index"))?;
            if idx >= counts.len() {
                return Err(ApiError::invalid_input(
                    "metadata references image index that was not uploaded",
                ));
            }
            counts[idx] = counts[idx]
                .checked_add(1)
                .ok_or_else(|| ApiError::invalid_input("image reference count overflow"))?;
            Ok(())
        };
        let create_cover_index = metadata
            .cover_image_indexes
            .first()
            .copied()
            .or_else(|| metadata.art_image_indexes.first().copied())
            .or_else(|| {
                metadata
                    .chapters
                    .iter()
                    .flat_map(|chapter| chapter.versions.iter())
                    .flat_map(|version| version.image_indexes.iter().copied())
                    .next()
            })
            .ok_or_else(|| ApiError::invalid_input("restore metadata has no images"))?;

        let mut image_ref_counts = vec![0usize; image_ids.len()];
        for index in metadata.cover_image_indexes.iter().copied() {
            register_ref(index, &mut image_ref_counts)?;
        }
        for index in metadata.art_image_indexes.iter().copied() {
            register_ref(index, &mut image_ref_counts)?;
        }
        for chapter in &metadata.chapters {
            for version in &chapter.versions {
                for index in version.image_indexes.iter().copied() {
                    register_ref(index, &mut image_ref_counts)?;
                }
            }
        }
        if metadata.cover_image_indexes.is_empty() {
            register_ref(create_cover_index, &mut image_ref_counts)?;
        }
        if image_ref_counts.iter().any(|count| *count == 0) {
            return Err(ApiError::invalid_input(
                "upload response contains image ids not referenced by metadata",
            ));
        }

        let mut prepared_file_ids = vec![Vec::<String>::new(); image_ids.len()];
        for (index, count) in image_ref_counts.into_iter().enumerate() {
            let bytes = self
                .fs
                .take_bytes(FileId::new(image_ids[index].clone()))
                .await?;
            let mut file_ids = Vec::with_capacity(count);
            for _ in 0..count {
                file_ids.push(self.upload_bytes_as_file_id(&bytes).await?);
            }
            prepared_file_ids[index] = file_ids;
        }
        let mut take_prepared_file_id = |image_index: u32| -> ApiResult<String> {
            let idx = usize::try_from(image_index)
                .map_err(|_| ApiError::invalid_input("invalid image index"))?;
            prepared_file_ids
                .get_mut(idx)
                .and_then(|values| values.pop())
                .ok_or_else(|| ApiError::invalid_input("missing referenced image payload"))
        };

        let cover_image_temp_name = take_prepared_file_id(create_cover_index)?;
        let create_request = Self::build_restore_create_request(&metadata, cover_image_temp_name);
        let manga_id = self.create(create_request, uid).await?;
        let cover_indexes = metadata.cover_image_indexes.clone();
        let art_indexes = metadata.art_image_indexes.clone();
        let chapters = metadata.chapters.clone();
        let volumes = metadata.volumes.clone();
        let visibility_value = metadata.visibility;

        for (index, image_index) in cover_indexes.iter().copied().enumerate() {
            if index == 0 {
                continue;
            }
            let file_id = take_prepared_file_id(image_index)?;
            self.add_cover(&manga_id, &file_id).await?;
        }
        for image_index in art_indexes.iter().copied() {
            let file_id = take_prepared_file_id(image_index)?;
            self.add_art(&manga_id, &file_id).await?;
        }

        let chapter_actions = ChapterActions {
            chapters: self.chapters.clone(),
            tags: self.tags.clone(),
            versions: self.versions.clone(),
            chapter_versions: self.chapter_versions.clone(),
            mangas: self.mangas.clone(),
            pages: self.pages.clone(),
            fs: self.fs.clone(),
        };

        for chapter in chapters {
            if !chapter.chapter.is_finite() || chapter.chapter < 0.0 {
                return Err(ApiError::invalid_input(
                    "restore metadata contains invalid chapter number",
                ));
            }
            if chapter.versions.is_empty() {
                return Err(ApiError::invalid_input(
                    "restore metadata contains chapter without versions",
                ));
            }

            let chapter_titles = {
                let titles = sanitize_non_empty(chapter.titles.clone());
                if titles.is_empty() {
                    vec![format!("Chapter {}", chapter.chapter)]
                } else {
                    titles
                }
            };
            let chapter_tags = chapter
                .tags
                .clone()
                .into_iter()
                .map(global_tag_from_export)
                .collect::<Vec<_>>();
            let chapter_sources = sanitize_non_empty(chapter.sources.clone());
            let chapter_release_date = parse_release_date(chapter.release_date.clone());

            for (version_index, version) in chapter.versions.into_iter().enumerate() {
                let version_name = {
                    let name = version.version.trim();
                    if name.is_empty() {
                        format!("restored-v{}", version_index + 1)
                    } else {
                        name.to_owned()
                    }
                };
                if version.image_indexes.is_empty() {
                    return Err(ApiError::invalid_input(
                        "restore metadata contains empty chapter version pages",
                    ));
                }
                let mut version_images = Vec::with_capacity(version.image_indexes.len());
                for image_index in version.image_indexes {
                    version_images.push(take_prepared_file_id(image_index)?);
                }

                chapter_actions
                    .add(
                        &manga_id,
                        if version_index == 0 {
                            chapter_titles.clone()
                        } else {
                            vec![]
                        },
                        chapter.chapter,
                        &version_name,
                        version_images,
                        if version_index == 0 {
                            chapter_tags.clone()
                        } else {
                            vec![]
                        },
                        if version_index == 0 {
                            chapter_sources.clone()
                        } else {
                            vec![]
                        },
                        if version_index == 0 {
                            chapter_release_date.clone()
                        } else {
                            None
                        },
                    )
                    .await?;
            }
        }

        if !volumes.is_empty() {
            self.set_volume_range(
                &manga_id,
                volumes
                    .into_iter()
                    .map(|v| VolumeRange {
                        start: v.start,
                        end: v.end,
                        title: v.title,
                    })
                    .collect(),
            )
            .await?;
        }
        let visibility = Visibility::try_from(visibility_value).unwrap_or(Visibility::Visible);
        if visibility != Visibility::Visible {
            self.mangas.set_visibility(&manga_id, visibility).await?;
        }

        Ok(())
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

    pub async fn create(&self, data: AddMangaRequest, uid: &str) -> ApiResult<String> {
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
            characters: vec![],
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
        Ok(mid.thing.id().to_string())
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
