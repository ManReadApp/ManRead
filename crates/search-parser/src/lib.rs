mod display;
mod shape;
mod to_json;

use std::{
    collections::{HashMap, VecDeque},
    fmt::{Display, Formatter},
    mem,
    str::FromStr,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

use shape::{Array, Item, ItemData, ItemOrArray, ItemValue};
use to_json::ToJson;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn parse_query(item: &str, or_default: bool) -> Result<String, String> {
    parse_str(item, or_default, &HashMap::new()).map(|v| v.item.to_json())
}

pub struct WPos<T: PartialEq + Eq + Display + ToJson> {
    pos: Pos,
    item: T,
}

impl ToJson for String {
    fn to_json(&self) -> String {
        format!("\"{self}\"")
    }
}

impl<T: PartialEq + Eq + Display + ToJson> WPos<T> {
    fn new(pos: Pos, item: T) -> Self {
        Self { pos, item }
    }
    fn nnew(pos: usize, item: T) -> Self {
        Self {
            pos: Pos {
                start: pos,
                end: pos + 1,
            },
            item,
        }
    }
}

impl Pos {
    fn join(&self, other: Pos) -> Pos {
        Pos {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Vec<WPos<Token>>> for WPos<Token> {
    fn from(mut value: Vec<WPos<Token>>) -> Self {
        let item = value.remove(0);
        let mut chars = vec![];
        let mut pos = item.pos;
        chars.push(item.item.to_string());
        for item in value {
            pos = pos.join(item.pos);
            chars.push(item.item.to_string());
        }
        WPos {
            pos,
            item: Token::Word(chars.join("")),
        }
    }
}

#[derive(PartialEq, Eq)]
enum Token {
    // step1 only
    QuotationMark,
    Apostrophe,
    Char(char),
    Escape,
    // step1 & step2
    /// ' '
    Space,
    /// :
    Colon,
    /// !
    ExclamationMark,
    /// (
    Open,
    /// )
    Close,
    Or,
    Eq,
    Smaller,
    Bigger,
    And,

    // step2
    Word(String),
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Pos {
    start: usize,
    end: usize,
}

impl PartialEq for WPos<Token2> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl Eq for WPos<Token2> {}

#[derive(Eq, PartialEq)]
enum Token2 {
    /// ' '
    Space,
    /// :
    Colon,
    /// !
    ExclamationMark,
    /// |
    Or,
    /// &
    And,
    /// =
    Eq,
    /// <
    Smaller,
    /// >
    Bigger,
    Word(String),
    Group((Vec<WPos<Token2>>, Pos)),
}

fn join(token: Option<WPos<String>>, s: &str, pos: Pos) -> WPos<String> {
    match token {
        Some(mut token) => {
            token.item.push_str(s);
            token.pos = token.pos.join(pos);
            token
        }
        None => WPos::new(pos, s.to_owned()),
    }
}

impl ToJson for Token2 {
    fn to_json(&self) -> String {
        unreachable!()
    }
}

impl ToJson for Token {
    fn to_json(&self) -> String {
        unreachable!()
    }
}

impl Display for Token2 {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

fn recursion(tokens: &mut impl Iterator<Item = WPos<Token>>) -> (Vec<WPos<Token2>>, Option<Pos>) {
    let mut items = vec![];
    while let Some(token) = tokens.next() {
        match token.item {
            Token::Space => items.push(WPos::new(token.pos, Token2::Space)),
            Token::Colon => items.push(WPos::new(token.pos, Token2::Colon)),
            Token::ExclamationMark => items.push(WPos::new(token.pos, Token2::ExclamationMark)),
            Token::Open => {
                let (tokens, end_pos) = recursion(tokens);
                let pos = Pos {
                    start: token.pos.start,
                    end: tokens
                        .iter()
                        .map(|v| v.pos.end)
                        .max()
                        .unwrap_or(token.pos.end),
                };
                items.push(WPos::new(
                    pos,
                    Token2::Group((tokens, token.pos.join(end_pos.unwrap_or(token.pos)))),
                ));
            }
            Token::Close => return (items, Some(token.pos)),
            Token::Or => items.push(WPos::new(token.pos, Token2::Or)),
            Token::Eq => items.push(WPos::new(token.pos, Token2::Eq)),
            Token::Smaller => items.push(WPos::new(token.pos, Token2::Smaller)),
            Token::Bigger => items.push(WPos::new(token.pos, Token2::Bigger)),
            Token::And => items.push(WPos::new(token.pos, Token2::And)),
            Token::Word(v) => items.push(WPos::new(token.pos, Token2::Word(v))),
            _ => unreachable!("already removed"),
        }
    }
    (items, None)
}

impl From<Vec<Token>> for Token {
    fn from(value: Vec<Token>) -> Self {
        Token::Word(value.into_iter().map(|v| v.to_string()).collect::<String>())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Char(c) => write!(f, "{c}"),
            Token::Word(v) => write!(f, "{}", v),
            _ => write!(
                f,
                "{}",
                match self {
                    Token::QuotationMark => "\"",
                    Token::Apostrophe => "'",
                    Token::Escape => "\\",
                    Token::Space => " ",
                    Token::Colon => ":",
                    Token::ExclamationMark => "!",
                    Token::Open => "(",
                    Token::Close => ")",
                    Token::Eq => "=",
                    Token::Smaller => "<",
                    Token::Bigger => ">",
                    Token::And => "&",
                    Token::Char(c) => unreachable!("{c}"),
                    Token::Word(v) => unreachable!("{v:?}"),
                    Token::Or => "|",
                }
            ),
        }
    }
}

pub fn parse_str(
    s: &str,
    or_default: bool,
    kind_map: &HashMap<String, ItemKind>,
) -> Result<WPos<Array>, String> {
    let tokens = s
        .chars()
        .enumerate()
        .map(|(i, c)| match c {
            '"' => WPos::nnew(i, Token::QuotationMark),
            '\'' => WPos::nnew(i, Token::Apostrophe),
            '\\' => WPos::nnew(i, Token::Escape),
            ' ' => WPos::nnew(i, Token::Space),
            ':' => WPos::nnew(i, Token::Colon),
            '!' => WPos::nnew(i, Token::ExclamationMark),
            '(' => WPos::nnew(i, Token::Open),
            ')' => WPos::nnew(i, Token::Close),
            '=' => WPos::nnew(i, Token::Eq),
            '<' => WPos::nnew(i, Token::Smaller),
            '>' => WPos::nnew(i, Token::Bigger),
            '&' => WPos::nnew(i, Token::And),
            '|' => WPos::nnew(i, Token::Or),
            _ => WPos::nnew(i, Token::Char(c)),
        })
        .collect::<Vec<_>>();
    let tokens = process_words(tokens);
    let (tokens, _) = recursion(&mut tokens.into_iter());
    tokens_to_array(
        kind_map,
        Pos {
            start: 0,
            end: s.len(),
        },
        tokens,
        or_default,
    )
}

enum States {
    None,
    StrBuilder(WPos<String>),
    StrNo(WPos<String>),
    Str(WPos<String>),
    ValueBuilder(WPos<String>, bool, Option<WPos<String>>),
    OrAnd(bool),
    OrAndNot(bool),
    Array {
        or: bool,
        not: bool,
        tokens: Vec<WPos<Token2>>,
        tokens_pos: Pos,
    },
    ArrayFinish(WPos<Array>),
}

fn tokens_to_array(
    kind_map: &HashMap<String, ItemKind>,
    range: Pos,
    tokens: Vec<WPos<Token2>>,
    or_default: bool,
) -> Result<WPos<Array>, String> {
    let mut tokens = tokens.into_iter().collect::<VecDeque<_>>();
    let mut states = States::None;

    let mut items = vec![];
    while let Some(token) = tokens.pop_front() {
        states = match states {
            States::None => match token.item {
                Token2::Space => States::None,
                Token2::Colon => States::StrBuilder(WPos::new(token.pos, ":".to_string())),
                Token2::ExclamationMark => {
                    States::StrBuilder(WPos::new(token.pos, "!".to_string()))
                }
                Token2::Or => States::OrAnd(true),
                Token2::And => States::OrAnd(false),
                Token2::Eq => States::StrBuilder(WPos::new(token.pos, "=".to_string())),
                Token2::Smaller => States::StrBuilder(WPos::new(token.pos, "<".to_string())),
                Token2::Bigger => States::StrBuilder(WPos::new(token.pos, ">".to_string())),
                Token2::Word(s) => States::StrBuilder(WPos::new(token.pos, s)),
                Token2::Group((tokens, pos)) => States::Array {
                    not: false,
                    or: or_default,
                    tokens,
                    tokens_pos: pos,
                },
            },
            States::StrBuilder(build) => match token.item {
                Token2::Space => States::Str(build),
                Token2::Colon => States::ValueBuilder(build, false, None),
                Token2::ExclamationMark => States::StrNo(build),
                Token2::Or => {
                    items.push(ItemOrArray::Item(WPos::new(
                        build.pos,
                        Item {
                            not: false,
                            or_post: Some(true),
                            data: ItemData {
                                name: build,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::None
                }
                Token2::And => {
                    items.push(ItemOrArray::Item(WPos::new(
                        build.pos,
                        Item {
                            not: false,
                            or_post: Some(false),
                            data: ItemData {
                                name: build,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::None
                }
                Token2::Eq => States::StrBuilder(join(Some(build), "=", token.pos)),
                Token2::Smaller => States::StrBuilder(join(Some(build), "<", token.pos)),
                Token2::Bigger => States::StrBuilder(join(Some(build), ">", token.pos)),
                Token2::Word(w) => States::StrBuilder(join(Some(build), w.as_str(), token.pos)),
                Token2::Group((tokens, tokens_pos)) => {
                    items.push(ItemOrArray::Item(WPos::new(
                        build.pos,
                        Item {
                            not: false,
                            or_post: None,
                            data: ItemData {
                                name: build,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::Array {
                        not: false,
                        or: or_default,
                        tokens,
                        tokens_pos,
                    }
                }
            },
            States::StrNo(s) => match token.item {
                Token2::Space => Err("unexpected Space".to_owned())?,
                Token2::Colon => States::ValueBuilder(s, true, None),
                Token2::ExclamationMark => Err("unexpected ExclamationMark".to_owned())?,
                Token2::Or => Err("unexpected Or".to_owned())?,
                Token2::And => Err("unexpected And".to_owned())?,
                Token2::Eq => Err("unexpected Eq".to_owned())?,
                Token2::Smaller => Err("unexpected Smaller".to_owned())?,
                Token2::Bigger => Err("unexpected Bigger".to_owned())?,
                Token2::Word(_) => Err("unexpected Word".to_owned())?,
                Token2::Group(_) => Err("unexpected group".to_owned())?,
            },
            States::Str(s) => match token.item {
                Token2::Space => States::Str(s),
                Token2::Colon => Err("unexpected Colon".to_owned())?,
                Token2::ExclamationMark => Err("unexpected ExclamationMark".to_owned())?,
                Token2::Or => {
                    items.push(ItemOrArray::Item(WPos::new(
                        s.pos,
                        Item {
                            not: false,
                            or_post: Some(true),
                            data: ItemData {
                                name: s,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::None
                }
                Token2::And => {
                    items.push(ItemOrArray::Item(WPos::new(
                        s.pos,
                        Item {
                            not: false,
                            or_post: Some(false),
                            data: ItemData {
                                name: s,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::None
                }
                Token2::Eq => Err("unexpected Eq".to_owned())?,
                Token2::Smaller => Err("unexpected Smaller".to_owned())?,
                Token2::Bigger => Err("unexpected Bigger".to_owned())?,
                Token2::Word(w) => {
                    items.push(ItemOrArray::Item(WPos::new(
                        s.pos,
                        Item {
                            not: false,
                            or_post: None,
                            data: ItemData {
                                name: s,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::StrBuilder(WPos::new(token.pos, w))
                }
                Token2::Group((tokens, tokens_pos)) => {
                    items.push(ItemOrArray::Item(WPos::new(
                        s.pos,
                        Item {
                            not: false,
                            or_post: None,
                            data: ItemData {
                                name: s,
                                value: ItemValue::None,
                            },
                        },
                    )));
                    States::Array {
                        or: or_default,
                        not: false,
                        tokens,
                        tokens_pos,
                    }
                }
            },
            States::OrAnd(v) => match token.item {
                Token2::Space => Err("unexpected Space".to_owned())?,
                Token2::Colon => Err("unexpected Colon".to_owned())?,
                Token2::ExclamationMark => States::OrAndNot(v),
                Token2::Group((tokens, tokens_pos)) => States::Array {
                    not: false,
                    or: v,
                    tokens,
                    tokens_pos,
                },
                Token2::Or => Err("unexpected Or".to_owned())?,
                Token2::And => Err("unexpected And".to_owned())?,
                Token2::Eq => Err("unexpected Eq".to_owned())?,
                Token2::Smaller => Err("unexpected Smaller".to_owned())?,
                Token2::Bigger => Err("unexpected Bigger".to_owned())?,
                Token2::Word(_) => Err("unexpected Word".to_owned())?,
            },
            States::OrAndNot(v) => match token.item {
                Token2::Space => Err("unexpected Space".to_owned())?,
                Token2::Colon => Err("unexpected Colon".to_owned())?,
                Token2::ExclamationMark => Err("unexpected ExclamationMark".to_owned())?,
                Token2::Group((tokens, tokens_pos)) => States::Array {
                    not: true,
                    or: v,
                    tokens,
                    tokens_pos,
                },
                Token2::Or => Err("unexpected Or".to_owned())?,
                Token2::And => Err("unexpected And".to_owned())?,
                Token2::Eq => Err("unexpected Eq".to_owned())?,
                Token2::Smaller => Err("unexpected Smaller".to_owned())?,
                Token2::Bigger => Err("unexpected Bigger".to_owned())?,
                Token2::Word(_) => Err("unexpected Word".to_owned())?,
            },
            States::Array {
                or,
                not,
                tokens,
                tokens_pos,
            } => {
                let mut array = tokens_to_array(kind_map, tokens_pos, tokens, or_default)?;
                array.item.or = or;
                array.item.not = not;
                States::ArrayFinish(array)
            }
            States::ArrayFinish(mut arr) => match token.item {
                Token2::Or => {
                    arr.item.or_post = Some(true);
                    items.push(ItemOrArray::Array(arr));
                    States::None
                }
                Token2::And => {
                    arr.item.or_post = Some(false);
                    items.push(ItemOrArray::Array(arr));
                    States::None
                }
                token_ => {
                    items.push(ItemOrArray::Array(arr));
                    tokens.push_front(WPos::new(token.pos, token_));
                    States::None
                }
            },
            States::ValueBuilder(key, not, value) => match token.item {
                Token2::Space => States::ValueBuilder(key, not, value),
                Token2::Colon => Err("unexpected Colon".to_owned())?,
                Token2::ExclamationMark => Err("unexpected ExclamationMark".to_owned())?,
                Token2::Or => Err("unexpected Or".to_owned())?,
                Token2::And => Err("unexpected And".to_owned())?,
                Token2::Eq => States::ValueBuilder(key, not, Some(join(value, "=", token.pos))),
                Token2::Smaller => {
                    States::ValueBuilder(key, not, Some(join(value, "<", token.pos)))
                }
                Token2::Bigger => States::ValueBuilder(key, not, Some(join(value, ">", token.pos))),
                Token2::Word(w) => States::ValueBuilder(key, not, Some(join(value, &w, token.pos))),
                Token2::Group((tokens, tokens_pos)) => {
                    items.push(ItemOrArray::Item(WPos::new(
                        value
                            .as_ref()
                            .map(|v| v.pos.join(key.pos))
                            .unwrap_or(key.pos),
                        Item {
                            not,
                            or_post: None,
                            data: ItemData {
                                value: match value {
                                    Some(v) => kind_map
                                        .get(&key.item)
                                        .copied()
                                        .unwrap_or_default()
                                        .parse(&v.item)?,
                                    None => ItemValue::None,
                                },
                                name: key,
                            },
                        },
                    )));
                    States::Array {
                        or: or_default,
                        not: false,
                        tokens,
                        tokens_pos,
                    }
                }
            },
        };
    }

    match states {
        States::None => {}
        States::StrBuilder(wpos) => items.push(ItemOrArray::Item(WPos::new(
            wpos.pos,
            Item {
                not: false,
                or_post: None,
                data: ItemData {
                    name: wpos,
                    value: ItemValue::None,
                },
            },
        ))),
        States::StrNo(wpos) => items.push(ItemOrArray::Item(WPos::new(
            wpos.pos,
            Item {
                not: true,
                or_post: None,
                data: ItemData {
                    name: wpos,
                    value: ItemValue::None,
                },
            },
        ))),
        States::Str(wpos) => items.push(ItemOrArray::Item(WPos::new(
            wpos.pos,
            Item {
                not: false,
                or_post: None,
                data: ItemData {
                    name: wpos,
                    value: ItemValue::None,
                },
            },
        ))),
        States::ValueBuilder(wpos, not, wpos1) => items.push(ItemOrArray::Item(WPos::new(
            wpos.pos,
            Item {
                not,
                or_post: None,
                data: ItemData {
                    value: match wpos1 {
                        Some(v) => kind_map
                            .get(&wpos.item)
                            .copied()
                            .unwrap_or_default()
                            .parse(&v.item)?,
                        None => ItemValue::None,
                    },
                    name: wpos,
                },
            },
        ))),
        States::OrAnd(_) => {}
        States::OrAndNot(_) => {}
        States::Array {
            or,
            not,
            tokens,
            tokens_pos,
        } => {
            let mut array = tokens_to_array(kind_map, tokens_pos, tokens, or_default)?;
            array.item.or = or;
            array.item.not = not;
            items.push(ItemOrArray::Array(array));
        }
        States::ArrayFinish(wpos) => items.push(ItemOrArray::Array(wpos)),
    }
    Ok(WPos::new(
        range,
        Array {
            or: or_default,
            not: false,
            or_post: None,
            items,
        },
    ))
}

fn process_words(tokens: Vec<WPos<Token>>) -> Vec<WPos<Token>> {
    let mut new = vec![];
    let mut builder = vec![];
    let mut open: Option<WPos<Token>> = None;
    let mut escape = false;
    for token in tokens {
        if let Some(open_) = &open {
            if &open_.item == &token.item {
                if escape {
                    escape = false;
                    builder.push(token);
                    continue;
                } else {
                    open = None;
                    builder.push(token);
                    let builder_new = mem::take(&mut builder);
                    new.push(WPos::from(builder_new));
                    continue;
                }
            } else {
                builder.push(token);
                continue;
            }
        } else {
            if escape {
                match token.item {
                    Token::Char(c) => {
                        builder.push(WPos::nnew(token.pos.start - 1, Token::Char('\\')));
                        builder.push(WPos::new(token.pos, Token::Char(c)));
                    }
                    Token::Word(_) => unreachable!(),
                    token_ => builder.push(WPos::new(token.pos, token_)),
                }
                escape = false;
            } else {
                match token.item {
                    Token::QuotationMark => {
                        builder.push(WPos::new(token.pos, Token::QuotationMark));
                        open = Some(WPos::new(token.pos, Token::QuotationMark));
                    }
                    Token::Apostrophe => {
                        builder.push(WPos::new(token.pos, Token::Apostrophe));
                        open = Some(WPos::new(token.pos, Token::Apostrophe));
                    }
                    Token::Char(c) => builder.push(WPos::new(token.pos, Token::Char(c))),
                    Token::Word(item) => builder.push(WPos::new(token.pos, Token::Word(item))),
                    Token::Escape => escape = true,
                    token_ => {
                        if !builder.is_empty() {
                            let builder_new = mem::take(&mut builder);
                            new.push(WPos::from(builder_new));
                        }
                        new.push(WPos::new(token.pos, token_));
                    }
                }
            }
        }
    }
    if !builder.is_empty() {
        new.push(WPos::from(builder));
    }
    new
}

/// define the type it should be parsed to
#[derive(Clone, Copy)]
pub enum ItemKind {
    Bool,
    Int,
    String,
    CmpFloat,
    CmpInt,
    Float,
}

impl Default for ItemKind {
    fn default() -> Self {
        ItemKind::String
    }
}

impl ItemKind {
    pub fn parse(&self, s: &str) -> Result<ItemValue, String> {
        Ok(match self {
            ItemKind::Bool => ItemValue::Bool(
                s.parse()
                    .map_err(|_| format!("Failed to parse: {} to bool", s))?,
            ),
            ItemKind::Int => ItemValue::Int(
                s.parse()
                    .map_err(|_| format!("Failed to parse: {} to Int", s))?,
            ),
            ItemKind::Float => ItemValue::Float(
                s.parse()
                    .map_err(|_| format!("Failed to parse: {} to Float", s))?,
            ),
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
