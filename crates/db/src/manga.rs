use std::collections::{HashMap, HashSet};

use api_structure::{
    search::{Array, Item, ItemOrArray, ItemValue, Order, SearchRequest},
    v1::{Status, TagSex, Visibility},
};
use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo,
    ThingArray,
};

use crate::{
    chapter::ChapterDBService,
    error::{DbError, DbResult},
    DB,
};

use super::{
    chapter::Chapter,
    kind::Kind,
    lists::MangaList,
    tag::{Empty, Tag},
    user::User,
    version::Version,
};

#[derive(SurrealSelect, Deserialize)]
pub struct MangaName {
    pub titles: HashMap<String, Vec<String>>,
}
#[derive(Deserialize)]
pub struct Count {
    count: u64,
}

#[derive(SurrealSelect, Deserialize)]
pub struct MangaTitle {
    /// Title map<language, title>
    pub titles: HashMap<String, Vec<String>>,
}
#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("mangas")]
#[sql(["DEFINE EVENT manga_updated ON TABLE mangas WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Manga {
    /// Title map<language, title>
    pub titles: HashMap<String, Vec<String>>,
    /// Id to kind. Kinds can be anything but are meant to be manga, manhwa, manhua, etc.
    pub kind: RecordIdType<Kind>,
    /// A description of the manga
    pub description: Option<String>,
    /// Tags that are associated with the manga as a whole
    #[serde(default = "vec_default")]
    pub tags: Vec<RecordIdType<Tag>>,
    /// Tags that are associated with the manga as a whole & seperate chapters. These tags will be generated each time a chapter is edited/deleted. Otherwise it will be updated
    pub generated_tags: Vec<RecordIdType<Tag>>,
    /// Status like completed, ongoing, etc.
    pub status: u64,
    /// Visibility like public, private, in review
    pub visibility: u64,
    /// Id to user who uploaded the manga
    pub uploader: RecordIdType<User>,
    /// Id to user who drew the manga
    pub artists: Vec<RecordIdType<User>>,
    /// Id to user who wrote the manga
    pub authors: Vec<RecordIdType<User>>,
    /// Extension of the cover. The path is {root}/covers/{manga_id}_{index}.{ext}
    /// Field will never be remved but set to none
    pub covers: Vec<Option<String>>,
    /// Ids to chapters ordered by chapter number
    pub chapters: Vec<RecordIdType<Chapter>>,
    /// references to other sites like tracker or fandom
    pub sources: Vec<String>,
    /// Relations to other manga(spinoffs, parodies, etc.)
    pub relations: Vec<RecordIdType<Manga>>,
    /// Scrapes data from other sites
    pub scraper: Vec<Scraper>,
    /// When the manga was updated in db
    #[opt(exclude = true)]
    pub updated: Datetime,
    /// When the manga was created in db
    #[opt(excludes = true)]
    pub created: Datetime,
    /// Extension of the cover. The path is {root}/art/{manga_id}_{index}.{ext}
    /// Field will never be remved but set to none
    pub art_ext: Vec<Option<String>>,
    /// If manga was published by a magazine or a publisher
    pub publishers: Vec<RecordIdType<User>>,
    /// The volumes is a display thing. There are no real volumes in the database. The numbering is a single number, but the volumes define a range where which name should be displayed
    pub volumes: Vec<Volume>,
}

impl PartialEq for Manga {
    fn eq(&self, _: &Self) -> bool {
        unreachable!("manga equality not implemented")
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Volume {
    pub title: Option<String>,
    pub start: f64,
    pub end: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Scraper {
    pub target: RecordIdType<Version>,
    #[serde(default = "true_default")]
    pub enabled: bool,
    pub url: String,
}

pub fn vec_default<T>() -> Vec<T> {
    Vec::new()
}

fn true_default() -> bool {
    true
}

#[derive(Default)]
pub struct MangaDBService {}

fn get_with_prio(v: HashMap<String, Vec<String>>, prio: &Vec<String>) -> String {
    for prio in prio {
        if let Some(v) = v.get(prio) {
            if let Some(v) = v.first() {
                return v.to_owned();
            }
        }
    }
    v.values()
        .next()
        .and_then(|v| v.first().cloned())
        .unwrap_or_default()
}

fn filter_array_or_item(ior: ItemOrArray) -> Option<ItemOrArray> {
    match ior {
        ItemOrArray::Array(array) => filter_array(array).map(ItemOrArray::Array),
        item => Some(item),
    }
}

fn filter_array(mut arr: Array) -> Option<Array> {
    arr.items = arr
        .items
        .into_iter()
        .filter_map(filter_array_or_item)
        .collect();
    if arr.items.is_empty() {
        return None;
    }
    Some(arr)
}

fn generate_item(
    item: Item,
    user_id: &RecordIdType<User>,
) -> Result<(String, Option<bool>), String> {
    let not = match item.not {
        true => "!",
        false => "",
    };
    let not2 = match item.not {
        true => "not",
        false => "",
    };
    let is_enum = matches!(item.data.value, ItemValue::None);
    let query = match (is_enum, item.data.name.as_str()) {
        (true, "next-available") => format!("reading.progress < 1 AND reading.progress != None"),
        (true, v) => format!(
            "array::flatten(object::values(titles)).any(|$s|string::contains(string::lowercase($s),'{}'))",
            v.to_lowercase()
        ),
        (false, "") | (false, "title") => format!(
            "array::flatten(object::values(titles)).any(|$s|string::contains(string::lowercase($s),'{}'))",
            item.data
                .value
                .get_string()
                .ok_or("title needs to be a string".to_owned())?.to_lowercase()
        ),
        (false, "description") => format!(
            "description {not}~ '{}'",
            item.data
                .value
                .get_string()
                .ok_or("description needs to be a string".to_owned())?
        ),
        (false,"k") |(false,"kind" )=> format!(
            "kind = (SELECT id FROM kinds WHERE kind {not}= '{}' LIMIT 1)[0].id",
            item.data
                .value
                .get_string()
                .ok_or("description needs to be a string".to_owned())?
        ),
        (false,"male")|(false, "m") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::Male as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"female") | (false,"f") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::Female as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"both" )| (false,"b") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::Both as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"male2female")|(false, "mf") =>{
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::MaleFemale as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"female2male" )| (false,"fm") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::FemaleMale as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"none") | (false,"n") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::None as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"unknown") | (false,"u" )=>{
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            let sex = TagSex::Unknown as u32;
            format!(
            "(SELECT id FROM tags WHERE tag = '{tag}' AND sex = {sex} LIMIT 1)[0].id {} in generated_tags",
            match item.not {
                true => "NOT",
                false => "",
            })
        }
        (false,"tag") |(false, "t") => {
            let tag = item
                .data
                .value
                .get_string()
                .ok_or("description needs to be a string, int".to_owned())?;
            format!(
            "generated_tags {} (SELECT id FROM tags WHERE tag = '{tag}').id",
            match item.not {
                true => "NONEINSIDE",
                false => "CONTAINSANY",
            })
        }
        (false,"status") | (false,"s") => format!(
            "status = {}",
            item.data
                .value
                .get_int()
                .ok_or("title needs to be a int".to_owned())?
        ),
        (false,"uploader") => format!("uploader {not}= (SELECT id FROM users WHERE array::some(names, |$n: string| string::lowercase($n) = '{}') LIMIT 1)[0].id ", item.data
            .value
            .get_string()
            .ok_or("uploader needs to be a string".to_owned())?),
        (false,"artist") => format!("(SELECT id FROM users WHERE array::some(names, |$n: string| string::lowercase($n) = '{}') LIMIT 1)[0].id {not2} in artists", item.data
            .value
            .get_string()
            .ok_or("artists needs to be a string".to_owned())?),
        (false,"author") | (false,"a")=> format!("(SELECT id FROM users WHERE array::some(names, |$n: string| string::lowercase($n) = '{}') LIMIT 1)[0].id {not2} in authors", item.data
            .value
            .get_string()
            .ok_or("authors needs to be a string".to_owned())?),
        (false,"publisher")|(false,"p" )=> format!("(SELECT id FROM users WHERE array::some(names, |$n: string| string::lowercase($n) = '{}') LIMIT 1)[0].id {not2} in publishers", item.data
            .value
            .get_string()
            .ok_or("publishers needs to be a string".to_owned())?),
       (false, "chapters") |(false,"c")=> {
            let (mut eq, mut bigger, number) = item.data.value.get_cmp_int()
                .ok_or("chapters needs to be a eg. >= 10".to_owned())?;
            if item.not {
                eq = !eq;
                bigger = !bigger;
            }
            format!("array::len(chapters) {}{} {number}", if bigger { ">" } else { "<" }, if eq { "=" } else { "" })
        }
        (false,"list") |(false, "l" )=> format!("id {not2} IN (SELECT mangas FROM {} WHERE name = '{}' AND user = {} LIMIT 1)", MangaList::name(),item.data
            .value
            .get_string()
            .ok_or("publishers needs to be a string".to_owned())?, user_id.to_string()),
        _ => Err(format!("Unknown item {}", item.data.name))?,
    };
    Ok((query, item.or_post))
}

fn generate_item_or_array(
    ior: ItemOrArray,
    user_id: &RecordIdType<User>,
) -> Result<(String, Option<bool>), String> {
    match ior {
        ItemOrArray::Item(item) => generate_item(item, user_id),
        ItemOrArray::Array(array) => generate_array(array, user_id),
    }
}

fn generate_array(
    arr: Array,
    user_id: &RecordIdType<User>,
) -> Result<(String, Option<bool>), String> {
    let mut queries = vec![];
    let len = arr.items.len();
    for (index, (query, or_post)) in arr
        .items
        .into_iter()
        .map(|v| generate_item_or_array(v, user_id))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .enumerate()
    {
        queries.push(query);
        if index != len - 1 {
            queries.push(
                match or_post.unwrap_or(arr.or) {
                    true => "OR",
                    false => "AND",
                }
                .to_owned(),
            );
        }
    }
    Ok((
        format!(
            "{}({})",
            match arr.not {
                true => "!",
                false => "",
            },
            queries.join(" "),
        ),
        arr.or_post,
    ))
}

fn search_array(q: &str, arr: &Array) -> bool {
    arr.items.iter().any(|v| match v {
        ItemOrArray::Item(item) => item.data.name.as_str() == q,
        ItemOrArray::Array(array) => search_array(q, array),
    })
}
impl MangaDBService {
    pub async fn add_cover(&self, mid: &str, ext: &str) -> DbResult<usize> {
        let id = RecordIdFunc::from((Manga::name(), mid));
        let _: Empty = id.clone().get(&*DB).await?.ok_or(DbError::NotFound)?;
        let v: Option<Manga> = id
            .patch(&*DB, PatchOp::add("/covers/-", Some(ext.to_owned())))
            .await?;
        Ok(v.unwrap().covers.len() - 1)
    }

    pub async fn remove_cover(&self, mid: &str, index: usize) -> DbResult<()> {
        let id = RecordIdFunc::from((Manga::name(), mid));
        let _: Empty = id.clone().get(&*DB).await?.ok_or(DbError::NotFound)?;
        let _: Option<Empty> = id
            .patch(
                &*DB,
                PatchOp::replace(&format!("/covers/{}", index), None::<String>),
            )
            .await?;
        Ok(())
    }

    pub async fn add_art(&self, mid: &str, ext: &str) -> DbResult<usize> {
        let id = RecordIdFunc::from((Manga::name(), mid));
        let _: Empty = id.clone().get(&*DB).await?.ok_or(DbError::NotFound)?;
        let v: Option<Manga> = id
            .patch(&*DB, PatchOp::add("/art_ext/-", Some(ext.to_owned())))
            .await?;
        Ok(v.unwrap().art_ext.len() - 1)
    }

    pub async fn remove_art(&self, mid: &str, index: usize) -> DbResult<()> {
        let id = RecordIdFunc::from((Manga::name(), mid));
        let _: Empty = id.clone().get(&*DB).await?.ok_or(DbError::NotFound)?;
        let _: Option<Empty> = id
            .patch(
                &*DB,
                PatchOp::replace(&format!("/art_ext/{}", index), None::<String>),
            )
            .await?;
        Ok(())
    }

    pub async fn scrapers(&self) -> DbResult<Vec<RecordIdType<Manga>>> {
        let items:Vec<RecordData<Empty>> = Manga::search(&*DB, Some(format!("WHERE array::len(array::filter(scraper, |$item| $item.enabled = true OR $item.enabled IS NONE)) > 0;"))).await?;
        Ok(items.into_iter().map(|v| v.id.into()).collect())
    }
    pub async fn get_names(
        &self,
        ids: impl Iterator<Item = RecordIdType<Manga>>,
        name_prio: Vec<String>,
    ) -> DbResult<Vec<(String, String)>> {
        let v: Vec<RecordData<MangaName>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get_part(&*DB)
            .await?;
        Ok(v.into_iter()
            .map(|v| {
                (
                    v.id.id().to_string(),
                    get_with_prio(v.data.titles, &name_prio),
                )
            })
            .collect())
    }
    pub async fn exists(&self, id: &str) -> DbResult<()> {
        let _: RecordData<Empty> = RecordIdFunc::from((Manga::name(), id))
            .get_part(&*DB)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(())
    }
    pub async fn add(&self, manga: Manga) -> DbResult<RecordIdType<Manga>> {
        manga
            .add(&*DB)
            .await?
            .map(|v| v.id.clone().into())
            .ok_or(DbError::NotFound)
    }

    pub async fn search(
        &self,
        data: SearchRequest,
        user: RecordIdType<User>,
        count: bool,
    ) -> DbResult<(u64, Vec<RecordData<Manga>>)> {
        let mut what = vec!["*"];
        let mut tb = Manga::name();
        let order = Order::try_from(data.order).unwrap();
        if search_array("next-available", &data.query) {
            tb = "(SELECT *, (select updated, progress from user_progress where user = users:1cstbe4i4bnvgoq7rue9 AND manga = $parent.id LIMIT 1)[0] as reading FROM mangas)";
        }

        match order {
            Order::ChapterCount => {
                what.push("array::len(chapters) as chapter_count");
            }
            Order::LastRead => {
                tb = "(SELECT *, (select updated, progress from user_progress where user = users:1cstbe4i4bnvgoq7rue9 AND manga = $parent.id LIMIT 1)[0] as reading FROM mangas)";
                what.push("reading");
            }
            Order::Alphabetical => {
                what.push("IF titles.en != NONE THEN titles.en[0] ELSE array::first(object::values(titles))[0] END AS title_en");
            }
            _ => {}
        }

        //TODO:
        // future: sources,relations
        // custom visibility
        let order_by = format!(
            "ORDER BY {} {}",
            match order {
                Order::Alphabetical => "title_en",
                Order::Created => "created",
                Order::Updated => "updated",
                Order::LastRead => "reading.updated",
                Order::Popularity => todo!(),
                Order::Random => "rand()",
                Order::Status => "status",
                Order::ChapterCount => "chapter_count",
            },
            match order {
                Order::Random => "",
                _ => match data.desc {
                    true => "DESC",
                    false => "ASC",
                },
            },
        );

        let limit = format!(
            "LIMIT {} START {}",
            data.limit,
            (data.page - 1) * data.limit,
        );
        let what = what.join(", ");
        let query_ = match filter_array(data.query) {
            Some(arr) => format!(
                "WHERE {}",
                generate_array(arr, &user)
                    .map_err(DbError::SearchParseError)?
                    .0
            ),
            None => "".to_owned(),
        };

        let query = format!("SELECT {what} FROM {tb} {} {order_by} {limit}", &query_);
        println!("{query}");
        let data: Vec<RecordData<Manga>> = DB.query(query).await?.take(0)?;

        if count {
            let count: Option<Count> = DB
                .query(format!("SELECT count() FROM {tb} {query_} GROUP ALL;"))
                .await?
                .take(0)?;
            Ok((count.map(|v| v.count).unwrap_or_default(), data))
        } else {
            Ok((0, data))
        }
    }

    pub async fn regenerate_tags(&self, id: &str) -> DbResult<()> {
        let id: RecordIdType<Manga> = RecordIdType::from((Manga::name(), id));
        let info = id.clone().get(&*DB).await?.ok_or(DbError::NotFound)?;
        let mut tags = info.data.tags;
        let chapters = info.data.chapters;
        tags.extend(ChapterDBService {}.get_tags(chapters).await?);
        let tags = tags
            .into_iter()
            .map(|v| v.thing.0)
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|v| RecordIdFunc::from(v))
            .collect::<Vec<_>>();
        id.patch(&*DB, PatchOp::replace("generated_tags", tags))
            .await?;
        Ok(())
    }

    pub async fn update(
        &self,
        id: &str,
        titles: HashMap<String, Vec<String>>,
        status: Status,
        description: Option<String>,
        tags: Vec<RecordIdType<Tag>>,
        authors: Vec<RecordIdFunc>,
        artists: Vec<RecordIdFunc>,
        publishers: Vec<RecordIdFunc>,
        sources: Vec<String>,
        scrapers: Vec<Scraper>,
        kind: RecordIdType<Kind>,
    ) -> DbResult<()> {
        let id = RecordIdFunc::from((Manga::name(), id));
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/titles", titles))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/description", description))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/tags", tags))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/authors", authors))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/artists", artists))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/publishers", publishers))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/kind", kind))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/scraper", scrapers))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/sources", sources))
            .await?;
        let _: Option<Empty> = id
            .clone()
            .patch(&*DB, PatchOp::replace("/status", status as u64))
            .await?;
        Ok(())
    }

    pub async fn set_visibility(&self, id: &str, visibility: Visibility) -> DbResult<()> {
        let _: Option<Empty> = RecordIdFunc::from((Manga::name(), id))
            .patch(&*DB, PatchOp::replace("/visibility", visibility as u64))
            .await?;
        Ok(())
    }

    pub async fn set_volumes(
        &self,
        id: &str,
        volumes: Vec<(Option<String>, f64, Option<f64>)>,
    ) -> DbResult<()> {
        let vols = volumes
            .into_iter()
            .map(|v| Volume {
                title: v.0,
                start: v.1,
                end: v.2,
            })
            .collect::<Vec<_>>();
        let _: Option<Empty> = RecordIdFunc::from((Manga::name(), id))
            .patch(&*DB, PatchOp::replace("/volumes", vols))
            .await?;
        Ok(())
    }

    pub async fn add_relation(&self, id: &str, relation_id: &str) -> DbResult<()> {
        let relation_id = RecordIdFunc::from((Manga::name(), relation_id));
        let _: Option<Empty> = RecordIdFunc::from((Manga::name(), id))
            .patch(&*DB, PatchOp::add("/relations", relation_id))
            .await?;
        Ok(())
    }

    pub async fn remove_relation(&self, id: &str, relation_id: &str) -> DbResult<()> {
        let info = self.get(id).await?;
        if let Some(v) = info
            .relations
            .iter()
            .position(|v| v.id().to_string() == relation_id)
        {
            let _: Option<Empty> = RecordIdFunc::from((Manga::name(), id))
                .patch(&*DB, PatchOp::remove(&format!("/relations/{}", v)))
                .await?;
        }

        Ok(())
    }

    pub async fn set_scraper(&self, id: &str, url: &str, value: bool) -> DbResult<()> {
        let info = self.get(id).await?;
        if let Some(v) = info.scraper.iter().position(|v| v.url == url) {
            let _: Option<Empty> = RecordIdFunc::from((Manga::name(), id))
                .patch(
                    &*DB,
                    PatchOp::replace(&format!("/scraper/{}/enabled", v), value),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn get(&self, id: &str) -> DbResult<Manga> {
        RecordIdFunc::from((Manga::name(), id))
            .get(&*DB)
            .await?
            .ok_or(DbError::NotFound)
    }
}
