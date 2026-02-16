use std::sync::Arc;

use api_structure::v1::{ChapterVersion, Claim, MangaReaderResponse, Page, ReaderChapter};
use db::{
    chapter::ChapterDBService, kind::KindDBService, lists::ListDBService, manga::MangaDBService,
    page::PageDBService, progress::UserProgressDBService, version_link::ChapterVersionDBService,
};

use crate::error::{ApiError, ApiResult};

pub struct ReaderActions {
    progresses: Arc<UserProgressDBService>,
    chapters: Arc<ChapterDBService>,
    pages: Arc<PageDBService>,
    chapter_versions: Arc<ChapterVersionDBService>,
    mangas: Arc<MangaDBService>,
    lists: Arc<ListDBService>,
    kinds: Arc<KindDBService>,
}
impl ReaderActions {
    pub async fn save_progress(
        &self,
        progress: f64,
        chapter_id: &str,
        claim: &Claim,
    ) -> ApiResult<()> {
        let progress = progress.clamp(0.0, 1.0);
        let manga_id = self.chapters.get_manga_id(chapter_id).await?;
        self.progresses
            .update(&claim.id, &manga_id, chapter_id, progress)
            .await?;
        if progress >= 0.95 {
            let _ = self
                .progresses
                .load_next_chapter(&claim.id, &manga_id, &chapter_id)
                .await;
        }
        Ok(())
    }

    pub async fn info(
        &self,
        manga_id: &str,
        chapter_id: Option<String>,
        claim: &Claim,
    ) -> ApiResult<MangaReaderResponse> {
        let manga = self.mangas.get(&manga_id).await?;
        let (chapter, progress) = match chapter_id {
            Some(v) => (v, 0.0_f64),
            None => match self
                .progresses
                .get_progress(&claim.id, manga_id)
                .await
                .map(|v| (v.0.id().to_string(), v.1))
            {
                Ok(v) => v,
                Err(_) => (
                    manga
                        .chapters
                        .get(0)
                        .ok_or(ApiError::NotFoundInDB)?
                        .id()
                        .to_string(),
                    0.0_f64,
                ),
            },
        };
        let mut chapters: Vec<ReaderChapter> = self
            .chapters
            .get_detail(manga.chapters.into_iter())
            .await?
            .into_iter()
            .map(|v| ReaderChapter {
                chapter_id: v.id.id().to_string(),
                titles: v.data.titles,
                chapter: v.data.chapter,
                sources: v.data.sources,
                release_date: v.data.release_date.map(|v| v.into_inner().0.to_rfc3339()),
                versions: v
                    .data
                    .versions
                    .into_iter()
                    .map(|v| (v.0, v.1.id().to_string()))
                    .collect(),
            })
            .collect();
        chapters.sort_by(|a, b| {
            a.chapter
                .partial_cmp(&b.chapter)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(MangaReaderResponse {
            favorite: self.lists.is_favorite(&manga_id, &claim.id).await,
            manga_id: manga_id.to_owned(),
            titles: manga
                .titles
                .into_iter()
                .map(|v| (v.0, v.1.into()))
                .collect(),
            kind: self.kinds.get_name(manga.kind).await?,
            description: manga.description,
            chapters,
            open_chapter: chapter,
            progress,
        })
    }

    pub async fn pages(&self, chapter_version_id: &str) -> ApiResult<ChapterVersion> {
        let info = self.chapter_versions.get(chapter_version_id).await?;
        let pages = self.pages.get(info.pages).await?;
        Ok(ChapterVersion {
            pages: pages
                .into_iter()
                .map(|v| {
                    (
                        v.data.page,
                        Page {
                            page: v.data.page,
                            id: v.id.id().to_string(),
                            width: v.data.width,
                            height: v.data.height,
                            ext: v.data.ext,
                        },
                    )
                })
                .collect(),
            link: info.link,
        })
    }
}
