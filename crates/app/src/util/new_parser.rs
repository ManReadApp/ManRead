use std::{fmt::Display, mem};

use api_structure::models::manga::search::{Array, Field, ItemData, ItemOrArray};
use log::warn;

// #[test]
// fn test() {
//     let str = r#"title\ this or "other title" and t:!"title about something" or t:what or("what"#;
//     let res = search_parser(
//         str,
//         false,
//         &vec![Field::new("title", vec!["", "t"], ItemKind::String)],
//     )
//     .unwrap();
//     println!("{:#?}", res)
// }

pub fn search_parser(s: &str, or_default: bool, infos: &Vec<Field>) -> Result<Array, String> {
    let tokens = lexar(s);
    let groups = post_lexar(tokens)?;
    let arr = extract2(groups)?;
    Ok(Array {
        not: false,
        or: or_default,
        or_post: None,
        items: arr
            .into_iter()
            .map(|v| v.validate(or_default, infos).unwrap())
            .collect(),
    })
}

impl ItemOr {
    fn validate(&self, or_default: bool, infos: &Vec<Field>) -> Option<ItemOrArray> {
        Some(match &self.data {
            Item::DetailItem { not, key, value } => {
                let v = infos.iter().find(|v| v.matches(&key))?;
                ItemOrArray::Item(api_structure::models::manga::search::Item {
                    not: *not,
                    or_post: self.or_post,
                    data: ItemData {
                        name: v.name.clone(),
                        value: v.kind.parse(value).unwrap(),
                    },
                })
            }
            Item::Item(value) => {
                let v = infos.iter().find(|v| v.matches(""))?;
                ItemOrArray::Item(api_structure::models::manga::search::Item {
                    not: false,
                    or_post: self.or_post,
                    data: ItemData {
                        name: v.name.clone(),
                        value: v.kind.parse(value).unwrap(),
                    },
                })
            }
            Item::Group { not, or, value } => ItemOrArray::Array(Array {
                or: or.clone().unwrap_or(or_default),
                not: *not,
                or_post: self.or_post,
                items: value
                    .iter()
                    .map(|v| v.validate(or_default, infos).unwrap())
                    .collect(),
            }),
        })
    }
}
/// chars to Tokens
fn lexar(s: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut token_builder = vec![];
    let build_token = |tokens: &mut Vec<Token>, token_builder: &mut Vec<char>| {
        if !token_builder.is_empty() {
            let mut new = vec![];
            mem::swap(&mut new, token_builder);
            tokens.push(Token::Keyword(new.into_iter().collect::<String>()));
        }
    };
    for char in s.chars() {
        match char {
            ' ' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::Space)
            }
            '\\' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::Escape)
            }
            '!' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::Not)
            }
            ':' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::What)
            }
            '\'' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::TextOpen1)
            }
            '"' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::TextOpen2)
            }
            '(' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::GroupOpen)
            }
            ')' => {
                build_token(&mut tokens, &mut token_builder);
                tokens.push(Token::GroupClose)
            }
            _ => token_builder.push(char),
        }
        build_token(&mut tokens, &mut token_builder);
    }
    tokens
}

/// Processes \, ", ' and generates hierachal Structure
fn post_lexar(tokens: Vec<Token>) -> Result<Vec<TokenGroup>, String> {
    let mut new_tokens = vec![];
    let mut next = false;
    for token in tokens {
        match token {
            Token::Escape => next = true,
            _ => match next {
                true => {
                    next = false;
                    let token = token
                        .to_char()
                        .ok_or("cannot escape lit".to_string())?
                        .to_string();
                    new_tokens.push(Token::Keyword(token));
                }
                false => new_tokens.push(token),
            },
        }
    }
    let mut tokens = vec![];
    let mut locked = None;
    for token in new_tokens {
        match &locked {
            Some(lock) => match &token == lock {
                true => {
                    locked = None;
                }
                false => {
                    let text = token.to_string();
                    tokens.push(Token::Keyword(text))
                }
            },
            None => match token {
                Token::TextOpen1 => locked = Some(Token::TextOpen1),
                Token::TextOpen2 => locked = Some(Token::TextOpen2),
                _ => {
                    tokens.push(token);
                }
            },
        }
    }
    let mut new_tokens = vec![];
    for token in tokens {
        match (new_tokens.last_mut(), &token) {
            (Some(Token::Keyword(a)), Token::Keyword(b)) => *a = format!("{a}{b}"),
            _ => new_tokens.push(token),
        }
    }
    Ok(extract(new_tokens))
}

/// hierachal structure & Token => LessToken
fn extract(input: Vec<Token>) -> Vec<TokenGroup> {
    let mut input = input.into_iter();
    let mut out = vec![];
    while let Some(v) = input.next() {
        match v {
            Token::Space => out.push(TokenGroup::Token(LessToken::Space)),
            Token::Not => out.push(TokenGroup::Token(LessToken::Not)),
            Token::What => out.push(TokenGroup::Token(LessToken::What)),
            Token::Keyword(v) => out.push(TokenGroup::Token(LessToken::Keyword(v))),
            Token::GroupOpen => {
                let mut collect = vec![];
                let mut count = 1;
                while let Some(token) = input.next() {
                    match token {
                        Token::GroupOpen => {
                            count += 1;
                        }
                        Token::GroupClose => {
                            count -= 1;
                        }
                        _ => {}
                    }
                    if count == 0 {
                        break;
                    }
                    collect.push(token)
                }
                println!("{:?}", collect);
                let group = extract(collect);
                out.push(TokenGroup::Group(group));
            }
            Token::GroupClose => {
                warn!("token ')' isnt supposed to be here")
            }
            Token::Escape | Token::TextOpen1 | Token::TextOpen2 => unreachable!(),
        }
    }
    out
}

/// Converts Vec<TokenGroup> to Vec<Item>
/// creates items for single lit, detail lit(key:(!)valeu), group
fn extract2(groups: Vec<TokenGroup>) -> Result<Vec<ItemOr>, String> {
    let mut res = vec![];
    let mut item_builder = ItemBuilder::default();
    for group in groups {
        match item_builder.step {
            Step::Beginning => match group {
                TokenGroup::Token(token) => match token {
                    LessToken::Space => {
                        //skip
                    }
                    LessToken::Not | LessToken::What => panic!("unexpected token: {:?}", token),
                    LessToken::Keyword(key) => {
                        item_builder.part1 = Some(key);
                        item_builder.step = Step::AfterFirstWord;
                    }
                },
                TokenGroup::Group(v) => res.push(ItemOr::new(Item::Group {
                    not: false,
                    or: None,
                    value: extract2(v)?,
                })),
            },
            Step::AfterFirstWord => match group {
                TokenGroup::Token(token) => match token {
                    LessToken::Space => {
                        let item = item_builder.build().unwrap();
                        res.push(ItemOr::new(item));
                    }
                    LessToken::Not => panic!("unexpected token: {:?}", token),
                    LessToken::What => {
                        item_builder.step = Step::AfterDouble;
                    }
                    LessToken::Keyword(_) => panic!("unreachable token: {:?}", token),
                },
                TokenGroup::Group(v) => {
                    let item = item_builder.build().unwrap();
                    res.push(ItemOr::new(item));
                    res.push(ItemOr::new(Item::Group {
                        not: false,
                        or: None,
                        value: extract2(v)?,
                    }))
                }
            },
            Step::AfterDouble => match group {
                TokenGroup::Token(token) => match token {
                    LessToken::Space | LessToken::What => {
                        panic!("unexpected token: {:?}", token)
                    }
                    LessToken::Not => item_builder.not = true,
                    LessToken::Keyword(token) => {
                        item_builder.part2 = Some(token);
                        let item = item_builder.build().unwrap();
                        res.push(ItemOr::new(item));
                    }
                },
                TokenGroup::Group(v) => {
                    let or = match item_builder.part1.as_ref().map(|v| v.as_str()) {
                        Some("or") => Some(true),
                        Some("and") => Some(false),
                        Some(v) => Err(format!("{v} is not a valid group prefix"))?,
                        None => None,
                    };
                    res.push(ItemOr::new(Item::Group {
                        not: item_builder.not,
                        or,
                        value: extract2(v)?,
                    }));
                    item_builder.reset();
                }
            },
        }
    }
    if let Some(v) = item_builder.build() {
        res.push(ItemOr::new(v));
    }
    let mut new: Vec<ItemOr> = vec![];
    for item in res {
        match (new.last_mut(), &item.is_or()) {
            (Some(v), Some(or)) => {
                if v.or_post.is_some() {
                    new.push(ItemOr {
                        data: Item::Item(
                            match or {
                                true => "or",
                                false => "and",
                            }
                            .to_string(),
                        ),
                        or_post: None,
                    })
                } else {
                    v.or_post = Some(*or);
                }
            }
            _ => new.push(item),
        }
    }
    Ok(new)
}

pub struct ItemOr {
    data: Item,
    or_post: Option<bool>,
}

impl ItemOr {
    pub fn new(data: Item) -> Self {
        Self {
            data,
            or_post: None,
        }
    }
}

pub enum Item {
    DetailItem {
        not: bool,
        key: String,
        value: String,
    },
    Item(String),
    Group {
        not: bool,
        or: Option<bool>,
        value: Vec<ItemOr>,
    },
}

impl ItemOr {
    fn is_or(&self) -> Option<bool> {
        match &self.data {
            Item::Item(v) => match v.as_str() {
                "or" => Some(true),
                "and" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Default)]
struct ItemBuilder {
    part1: Option<String>,
    not: bool,
    part2: Option<String>,
    step: Step,
}

impl ItemBuilder {
    pub fn build(&mut self) -> Option<Item> {
        let res = match self.step {
            Step::Beginning => None,
            Step::AfterFirstWord => Some(Item::Item(self.part1.clone().unwrap_or_default())),
            Step::AfterDouble => Some(Item::DetailItem {
                not: self.not,
                key: self.part1.clone().unwrap_or_default(),
                value: self.part2.clone().unwrap_or_default(),
            }),
        };
        self.reset();
        res
    }
    fn reset(&mut self) {
        self.not = false;
        self.part1 = None;
        self.part2 = None;
        self.step = Step::Beginning;
    }
}

#[derive(Default)]
enum Step {
    #[default]
    Beginning,
    AfterFirstWord,
    AfterDouble,
}

#[derive(PartialEq, Eq, Debug)]
enum Token {
    /// ' '
    Space,
    /// '\'
    Escape,
    /// '!'
    Not,
    /// ':'
    What,
    /// '''
    TextOpen1,
    /// '"'
    TextOpen2,
    /// '('
    GroupOpen,
    /// ')'
    GroupClose,
    Keyword(String),
}

#[derive(Debug)]
enum LessToken {
    /// ' '
    Space,
    /// '!'
    Not,
    /// ':'
    What,
    Keyword(String),
}

enum TokenGroup {
    Token(LessToken),
    Group(Vec<TokenGroup>),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Keyword(ref v) => v.clone(),
                _ => self.to_char().unwrap().to_string(),
            }
        )
    }
}

impl Token {
    fn to_char(&self) -> Option<char> {
        Some(match self {
            Token::Space => ' ',
            Token::Escape => '\\',
            Token::Not => '!',
            Token::What => ':',
            Token::TextOpen1 => '\'',
            Token::TextOpen2 => '"',
            Token::GroupOpen => '(',
            Token::GroupClose => ')',
            Token::Keyword(_) => None?,
        })
    }
}
