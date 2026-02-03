use std::fmt::{Display, Formatter};

use crate::{
    shape::{Array, Item, ItemOrArray, ItemValue},
    to_json::ToJson,
    WPos,
};

impl<T: Display + Eq + PartialEq + ToJson> Display for WPos<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.item.fmt(f)
    }
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

impl Display for ItemValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ItemValue::None => String::new(),
                ItemValue::Bool(bool) => bool.to_string(),
                ItemValue::Int(v) => v.to_string(),
                ItemValue::StrInt(a, b) => format!("{a}, {b}"),
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

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}:{}",
            self.data.name,
            match self.not {
                true => "!",
                false => "",
            },
            self.data.value
        )
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.or {
            true => "|(",
            false => "&(",
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
