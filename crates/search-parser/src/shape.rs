use crate::WPos;

pub enum ItemValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    StrInt(String, u64),
    String(String),
    CmpFloat { eq: bool, bigger: bool, value: f64 },
    CmpInt { eq: bool, bigger: bool, value: i64 },
}

pub struct ItemData {
    pub name: WPos<String>,
    pub value: ItemValue,
}

/// item include or exclude
pub struct Item {
    pub not: bool,
    pub or_post: Option<bool>,
    pub data: ItemData,
}

pub enum ItemOrArray {
    Item(WPos<Item>),
    Array(WPos<Array>),
}

pub struct Array {
    pub or: bool,
    pub not: bool,
    pub or_post: Option<bool>,
    pub items: Vec<ItemOrArray>,
}

impl PartialEq for Item {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Eq for Item {}

impl PartialEq for Array {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Eq for Array {}
