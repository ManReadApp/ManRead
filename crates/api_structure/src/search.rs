use crate::error::ApiErr;
use crate::ApiErrorType;
use crate::RequestImpl;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub trait DisplaySearch: DeserializeOwned + Send {
    fn image_number(&self) -> u32;
    fn internal(&self) -> bool;
    fn id_url(&self) -> &String;
    fn ext(&self) -> Cow<String>;
    fn status(&self) -> Cow<Status>;
    fn titles(&self) -> Cow<HashMap<String, Vec<String>>>;
    fn cover(&self) -> &str;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResponse {
    pub manga_id: String,
    pub titles: HashMap<String, Vec<String>>,
    pub tags: Vec<String>,
    pub status: Status,
    pub ext: String,
    pub number: u32,
}

impl DisplaySearch for SearchResponse {
    fn image_number(&self) -> u32 {
        self.number
    }

    fn internal(&self) -> bool {
        true
    }

    fn id_url(&self) -> &String {
        &self.manga_id
    }

    fn ext(&self) -> Cow<String> {
        Cow::Borrowed(&self.ext)
    }

    fn status(&self) -> Cow<Status> {
        Cow::Borrowed(&self.status)
    }

    fn titles(&self) -> Cow<HashMap<String, Vec<String>>> {
        Cow::Borrowed(&self.titles)
    }

    fn cover(&self) -> &str {
        ""
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Status {
    Dropped,
    Hiatus,
    Ongoing,
    Completed,
    Upcoming,
}

impl TryFrom<u64> for Status {
    type Error = ApiErr;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Dropped),
            1 => Ok(Self::Hiatus),
            2 => Ok(Self::Ongoing),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Upcoming),
            _ => Err(ApiErr {
                message: Some("Couldnt find manga status".to_string()),
                cause: None,
                err_type: ApiErrorType::InternalError,
            }),
        }
    }
}

impl From<Status> for u64 {
    fn from(value: Status) -> Self {
        match value {
            Status::Dropped => 0,
            Status::Hiatus => 1,
            Status::Ongoing => 2,
            Status::Completed => 3,
            Status::Upcoming => 4,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    pub order: Order,
    pub desc: bool,
    pub limit: u32,
    pub page: u32,
    pub query: ItemOrArray,
}

impl RequestImpl for SearchRequest {
    const ROUTE: &'static str = "search";
    const AUTH: bool = true;
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    Created,
    Alphabetical,
    Updated,
    LastRead,
    Popularity,
    Random,
}

/// can contain item or array
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum ItemOrArray {
    Item(Item),
    Array(Array),
}

impl Display for ItemOrArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ItemOrArray::Item(v) => v.to_string(),
                ItemOrArray::Array(v) => v.to_string(),
            }
        )
    }
}

/// array joined with and or or
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Array {
    pub or: bool,
    pub items: Vec<ItemOrArray>,
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.or {
            true => "or:(",
            false => "and:(",
        };
        write!(
            f,
            "{prefix}{})",
            self.items
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

/// item include or exclude
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Item {
    pub not: bool,
    pub data: ItemData,
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}{}",
            self.data.name,
            match self.not {
                true => "!",
                false => "",
            },
            self.data.value
        )
    }
}

impl Item {
    pub fn new(data: ItemData) -> Self {
        Self { not: false, data }
    }

    pub fn new_exclude(data: ItemData) -> Self {
        Self { not: true, data }
    }
}

/// field and value
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ItemData {
    pub name: String,
    pub value: ItemValue,
}

impl ItemData {
    pub fn enum_(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            value: ItemValue::None,
        }
    }
}

/// define the type it should be parsed to
pub enum ItemKind {
    Bool,
    Int,
    String,
    CmpFloat,
    CmpInt,
    Float,
}

impl ItemKind {
    pub fn parse(&self, s: &str) -> Result<ItemValue, String> {
        Ok(match self {
            ItemKind::Bool => ItemValue::Bool(s.parse().map_err(|_| "")?),
            ItemKind::Int => ItemValue::Int(s.parse().map_err(|_| "")?),
            ItemKind::Float => ItemValue::Float(s.parse().map_err(|_| "")?),
            ItemKind::String => ItemValue::String(s.to_string()),
            ItemKind::CmpFloat => {
                let (bigger, eq, value) = parse(s)?;
                ItemValue::CmpFloat { eq, bigger, value }
            }
            ItemKind::CmpInt => {
                let (bigger, eq, value) = parse(s)?;
                ItemValue::CmpInt { eq, bigger, value }
            }
        })
    }
}

/// enum with different values
#[derive(Serialize, Deserialize, Debug)]
pub enum ItemValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    CmpFloat { eq: bool, bigger: bool, value: f32 },
    CmpInt { eq: bool, bigger: bool, value: i64 },
}

/// mostly auto generated
impl PartialEq for ItemValue {
    #[inline]
    fn eq(&self, other: &ItemValue) -> bool {
        let self_tag = core::mem::discriminant(self);
        let other_tag = core::mem::discriminant(other);
        self_tag == other_tag
            && match (self, other) {
                (ItemValue::Bool(self_value), ItemValue::Bool(other_value)) => {
                    *self_value == *other_value
                }
                (ItemValue::Int(self_value), ItemValue::Int(other_value)) => {
                    *self_value == *other_value
                }
                (ItemValue::Float(self_value), ItemValue::Float(other_value)) => {
                    format!("{:.4}", self_value) == format!("{:.4}", other_value)
                }
                (ItemValue::String(self_value), ItemValue::String(other_value)) => {
                    *self_value == *other_value
                }
                (
                    ItemValue::CmpInt {
                        eq: self_eq,
                        bigger: self_bigger,
                        value: self_value,
                    },
                    ItemValue::CmpInt {
                        eq: other_eq,
                        bigger: other_bigger,
                        value: other_value,
                    },
                ) => {
                    *self_eq == *other_eq
                        && *self_bigger == *other_bigger
                        && *self_value == *other_value
                }
                (
                    ItemValue::CmpFloat {
                        eq: self_eq,
                        bigger: self_bigger,
                        value: self_value,
                    },
                    ItemValue::CmpFloat {
                        eq: other_eq,
                        bigger: other_bigger,
                        value: other_value,
                    },
                ) => {
                    *self_eq == *other_eq
                        && *self_bigger == *other_bigger
                        && format!("{:.4}", self_value) == format!("{:.4}", other_value)
                }
                _ => true,
            }
    }
}

impl Eq for ItemValue {}

impl Display for ItemValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ItemValue::None => String::new(),
                ItemValue::Bool(bool) => bool.to_string(),
                ItemValue::Int(v) => v.to_string(),
                ItemValue::Float(v) => v.to_string(),
                ItemValue::String(s) => format!("\"{s}\""),
                ItemValue::CmpFloat { eq, bigger, value } => {
                    format!(
                        "{}{}{}",
                        match bigger {
                            true => ">",
                            false => "<",
                        },
                        match eq {
                            true => "=",
                            false => "",
                        },
                        value
                    )
                }
                ItemValue::CmpInt { eq, bigger, value } => {
                    format!(
                        "{}{}{}",
                        match bigger {
                            true => ">",
                            false => "<",
                        },
                        match eq {
                            true => "=",
                            false => "",
                        },
                        value
                    )
                }
            }
        )
    }
}

pub struct Field {
    pub name: String,
    pub abbr: Vec<String>,
    pub kind: ItemKind,
}

impl Field {
    pub fn new(name: String, abbr: Vec<String>, kind: ItemKind) -> Self {
        Self { name, abbr, kind }
    }
}

fn parse<T: FromStr>(s: &str) -> Result<(bool, bool, T), String> {
    let (str, b, s) = if let Some(v) = s.strip_prefix('>') {
        (v, true, false)
    } else if let Some(v) = s.strip_prefix('<') {
        (v, false, true)
    } else {
        (s, false, false)
    };
    let (eq, num) = if let Some(v) = str.strip_prefix('=') {
        (true, v)
    } else {
        (false, str)
    };
    Ok((
        b == s || b,
        eq,
        num.parse::<T>()
            .map_err(|_| format!("Failed to parse: {}", num))?,
    ))
}
