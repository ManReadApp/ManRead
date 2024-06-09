use crate::error::ApiErr;
use crate::search::Status;
use crate::{ApiErrorType, RequestImpl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub struct MangaInfoRequest {
    pub manga_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct MangaInfoResponse {
    pub manga_id: String,
    pub titles: HashMap<String, Vec<String>>,
    pub kind: String,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub status: Status,
    pub visibility: Visibility,
    pub uploader: String,
    pub my: bool,
    pub artists: Vec<String>,
    pub authors: Vec<String>,
    pub cover: u32,
    pub cover_ext: String,
    pub chapters: Vec<Chapter>,
    pub sources: Vec<ExternalSite>,
    pub relations: Vec<(String, String)>,
    pub scraper: bool,
    pub favorite: bool,
    /// manga_id
    pub progress: Option<String>,
}

impl RequestImpl for MangaInfoRequest {
    const ROUTE: &'static str = "info";
    const AUTH: bool = true;
}

#[derive(Serialize, Deserialize)]
pub struct ExternalSite {
    pub url: String,
    pub icon_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<Tag>,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Tag {
    pub tag: String,
    pub description: Option<String>,
    pub sex: TagSex,
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.sex, self.tag)
    }
}

impl Display for TagSex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagSex::Female => write!(f, "♀"),
            TagSex::Male => write!(f, "♂"),
            TagSex::Both => write!(f, "⚤"), //⚥
            TagSex::None => write!(f, ""),
            TagSex::FemaleMale => write!(f, "♀→♂"),
            TagSex::MaleFemale => write!(f, "♂→♀"),
            TagSex::Unknown => write!(f, "⚧"),
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TagSex {
    Female = 0,
    Male = 1,
    Both = 2,
    None = 3,
    FemaleMale = 4,
    MaleFemale = 5,
    Unknown = 6,
}

impl TagSex {
    pub fn get_name(&self) -> &'static str {
        match self {
            TagSex::Female => "female",
            TagSex::Male => "male",
            TagSex::Both => "both",
            TagSex::None => "none",
            TagSex::FemaleMale => "female to male",
            TagSex::MaleFemale => "male to female",
            TagSex::Unknown => "i don't know",
        }
    }
    pub fn get_all() -> Vec<TagSex> {
        [
            TagSex::Female,
            TagSex::Male,
            TagSex::Both,
            TagSex::None,
            TagSex::FemaleMale,
            TagSex::MaleFemale,
            TagSex::Unknown,
        ]
        .to_vec()
    }
}

impl From<u64> for TagSex {
    fn from(value: u64) -> Self {
        match value {
            0 => TagSex::Female,
            1 => TagSex::Male,
            2 => TagSex::Both,
            3 => TagSex::None,
            4 => TagSex::FemaleMale,
            5 => TagSex::MaleFemale,
            _ => TagSex::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Visibility {
    /// Everyone
    Visible,
    /// Admins,Coadmins, Mods, and Author
    Hidden,
    /// Admins,Coadmins, Mods
    AdminReview,
}

impl TryFrom<u64> for Visibility {
    type Error = ApiErr;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Visible),
            1 => Ok(Self::Hidden),
            2 => Ok(Self::AdminReview),
            _ => Err(ApiErr {
                message: Some("unknown visibility".to_string()),
                cause: None,
                err_type: ApiErrorType::InternalError,
            }),
        }
    }
}
