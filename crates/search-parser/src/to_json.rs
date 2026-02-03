use std::fmt::Display;

use crate::{
    shape::{Array, Item, ItemData, ItemOrArray, ItemValue},
    WPos,
};

impl ToJson for Array {
    fn to_json(&self) -> String {
        format!(
            r#"{{"or": {}, "not":{}, "or_post":{}, "items":[{}]}}"#,
            self.or,
            self.not,
            self.or_post
                .map(|v| v.to_string())
                .unwrap_or("null".to_string()),
            self.items
                .iter()
                .map(|v| v.to_json())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl ToJson for Item {
    fn to_json(&self) -> String {
        format!(
            r#"{{"not":{}, "or_post":{}, "data": {}}}"#,
            self.not,
            self.or_post
                .map(|v| v.to_string())
                .unwrap_or("null".to_string()),
            self.data.to_json()
        )
    }
}

impl ToJson for ItemData {
    fn to_json(&self) -> String {
        format!(
            r#"{{"name": "{}", "name_pos": {{"start":{}, "end": {}}} ,"value": {}}}"#,
            self.name.item,
            self.name.pos.start,
            self.name.pos.end,
            self.value.to_json()
        )
    }
}

impl ToJson for ItemValue {
    fn to_json(&self) -> String {
        match self {
            ItemValue::None => "\"None\"".to_owned(),
            ItemValue::Bool(v) => v.to_string(),
            ItemValue::Int(v) => v.to_string(),
            ItemValue::Float(v) => v.to_string(),
            ItemValue::StrInt(a, b) => format!("[\"{a}\", {b}]"),
            ItemValue::String(s) => format!("\"{s}\""),
            ItemValue::CmpFloat { eq, bigger, value } => {
                format!(r#"{{"eq":{eq}, "bigger":{bigger}, "value":{value}}}"#)
            }
            ItemValue::CmpInt { eq, bigger, value } => {
                format!(r#"{{"eq":{eq}, "bigger":{bigger}, "value":{value}}}"#)
            }
        }
    }
}

impl ToJson for ItemOrArray {
    fn to_json(&self) -> String {
        match self {
            ItemOrArray::Item(wpos) => wpos.to_json(),
            ItemOrArray::Array(wpos) => wpos.to_json(),
        }
    }
}

pub trait ToJson {
    fn to_json(&self) -> String;
}

impl<T: Display + Eq + PartialEq + ToJson> WPos<T> {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"pos":{{"start":{}, "end":{}}}, {}"#,
            self.pos.start,
            self.pos.end,
            &self.item.to_json().trim()[1..]
        )
    }
}
