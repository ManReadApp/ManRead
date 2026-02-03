use std::fmt::{Display, Formatter};

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug, ApiComponent, JsonSchema)]
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

#[derive(
    Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Copy, Debug, ApiComponent, JsonSchema,
)]
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
