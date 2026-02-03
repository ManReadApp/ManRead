use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo,
};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::progress::UserProgress,
};

use super::{manga::Manga, tag::Empty, user::User};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("manga_lists")]
#[sql(["DEFINE EVENT manga_list_updated ON TABLE manga_lists WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct MangaList {
    /// Manga list name
    pub name: String,
    /// User who created the list
    pub user: RecordIdType<User>,
    /// List of mangas
    pub mangas: Vec<RecordIdType<Manga>>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(SurrealSelect, Deserialize)]
pub struct MangaListName {
    pub name: String,
}

#[derive(Default)]
pub struct ListDBService {}

async fn get_list(name: &str, user: &str) -> ApiResult<RecordData<MangaList>> {
    let user = RecordIdFunc::from((User::name(), user)).to_string();
    //TODO: name shouldnt contain escape characters, sql injection
    let mut v = MangaList::search(
        &*DB,
        Some(format!("WHERE name = '{name}' AND user = {user} LIMIT 1")),
    )
    .await?;
    if v.is_empty() {
        return Err(ApiError::NotFoundInDB);
    }
    Ok(v.remove(0))
}
impl ListDBService {
    pub async fn add(&self, name: &str, user: &str) -> ApiResult<()> {
        MangaList {
            name: name.to_owned(),
            user: RecordIdType::from((User::name(), user)),
            mangas: vec![],
            updated: Default::default(),
            created: Default::default(),
        }
        .add_i(&*DB)
        .await?;
        Ok(())
    }
    pub async fn add_manga(&self, name: &str, user: &str, manga: &str) -> ApiResult<()> {
        let manga = RecordIdFunc::from((Manga::name(), manga));
        let _: Option<RecordData<Empty>> = get_list(name, user)
            .await?
            .patch(&*DB, PatchOp::add("/mangas", manga))
            .await?;
        Ok(())
    }
    pub async fn remove_manga(&self, name: &str, user: &str, manga_id: &str) -> ApiResult<()> {
        let list = get_list(name, user).await?;
        let index = list
            .data
            .mangas
            .iter()
            .enumerate()
            .find(|v| v.1.id().to_string() == manga_id)
            .map(|v| v.0)
            .ok_or(ApiError::NotFoundInDB)?;
        let _: Option<RecordData<Empty>> = list
            .patch(&*DB, PatchOp::remove(&format!("/mangas/{index}")))
            .await?;
        Ok(())
    }
    pub async fn delete(&self, name: &str, user: &str) -> ApiResult<()> {
        get_list(name, user).await?.delete_s(&*DB).await?;
        Ok(())
    }

    pub async fn get(&self, user: &str) -> ApiResult<Vec<String>> {
        let user = RecordIdFunc::from((User::name(), user)).to_string();
        let m: Vec<RecordData<MangaListName>> =
            MangaList::search(&*DB, Some(format!("WHERE user = {user}"))).await?;
        Ok(m.into_iter().map(|v| v.data.name).collect())
    }
    pub async fn is_favorite(&self, manga_id: &str, user: &str) -> bool {
        let list = get_list("favorites", user).await;
        match list {
            Ok(list) => list
                .data
                .mangas
                .iter()
                .any(|v| v.id().to_string() == manga_id),
            Err(_) => false,
        }
    }

    pub async fn is_reading(&self, manga_id: &str, user: &str) -> bool {
        let user = RecordIdFunc::from((User::name(), user)).to_string();
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let v: Vec<RecordData<Empty>> = UserProgress::search(
            &*DB,
            Some(format!(
                "WHERE user = {user} AND manga = {manga_id} LIMIT 1"
            )),
        )
        .await
        .unwrap_or_default();
        !v.is_empty()
    }
}
