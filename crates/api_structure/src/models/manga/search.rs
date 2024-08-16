use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::error::{ApiErr, ApiErrorType};

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    Created,
    Alphabetical,
    Updated,
    LastRead,
    Popularity,
    Random,
}

impl TryFrom<String> for Order {
    type Error = ApiErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "created" => Self::Created,
            "alphabetical" => Self::Alphabetical,
            "updated" => Self::Updated,
            "last_read" => Self::LastRead,
            "popularity" => Self::Popularity,
            "random" => Self::Random,

            _ => Err(ApiErr {
                message: Some(format!("{value} is not a valid order")),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            })?,
        })
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Order::Created => "created",
                Order::Alphabetical => "alphabetical",
                Order::Updated => "updated",
                Order::LastRead => "last_read",
                Order::Popularity => "popularity",
                Order::Random => "random",
            }
        )
    }
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
    pub or_post: Option<bool>,
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
    pub or_post: Option<bool>,
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
        Self {
            not: false,
            or_post: None,
            data,
        }
    }

    pub fn new_exclude(data: ItemData) -> Self {
        Self {
            not: true,
            or_post: None,
            data,
        }
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
    CmpFloat { eq: bool, bigger: bool, value: f64 },
    CmpInt { eq: bool, bigger: bool, value: i64 },
}

impl ItemValue {
    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn get_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<f64> {
        match self {
            Self::Float(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            Self::String(i) => Some(i.clone()),
            _ => None,
        }
    }

    pub fn get_cmp_float(&self) -> Option<(bool, bool, f64)> {
        match self {
            Self::CmpFloat { eq, bigger, value } => Some((*eq, *bigger, *value)),
            _ => None,
        }
    }
    pub fn get_cmp_int(&self) -> Option<(bool, bool, i64)> {
        match self {
            Self::CmpInt { eq, bigger, value } => Some((*eq, *bigger, *value)),
            _ => None,
        }
    }

    pub fn get_id_or_value(&self) -> Option<IdOrValue> {
        let id = self.get_int();
        if let Some(v) = id {
            Some(IdOrValue::Id(v))
        } else if let Some(v) = self.get_string() {
            Some(IdOrValue::Value(v))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IdOrValue {
    Value(String),
    Id(i64),
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
