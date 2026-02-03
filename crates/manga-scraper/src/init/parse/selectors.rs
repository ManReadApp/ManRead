use regex::Regex;
use scraper::Selector;

use crate::InitError;

#[derive(Debug)]
pub struct Query {
    pub name: Option<String>,
    /// Query & or if true & and if false
    pub queries: Vec<(Vec<Kind>, bool)>,
    pub r_site: Option<Box<Query>>,
}

impl TryFrom<&str> for Query {
    type Error = InitError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut chars = s.chars();
        let mut name = vec![];
        loop {
            let c = match chars.next() {
                Some(c) => c,
                None => todo!("add error for this case"),
            };

            if c == '[' {
                break;
            }
            name.push(c);
        }

        let mut kind = vec![];
        loop {
            let c = match chars.next() {
                Some(c) => c,
                None => todo!("add error for this case"),
            };

            if c == ']' {
                break;
            }
            kind.push(c);
        }

        let str = chars.collect::<String>();
        let (left, right) = str
            .split_once("=>")
            .map(|v| (v.0, Some(v.1)))
            .unwrap_or((&str, None));
        let right = right.map(Query::try_from).transpose()?.map(Box::new);

        let mut seperator;
        let mut builder = vec![];
        let mut items = vec![];
        let mut kind = KindBuilder::try_from(kind.into_iter().collect::<String>().as_str())?;
        let mut kind_builder = false;
        let mut left = left.chars();
        while let Some(c) = left.next() {
            if matches!(kind, KindBuilder::Regex(_)) {
                builder.push(c);
                continue;
            }
            if kind_builder {
                if builder.is_empty() && c == '[' {
                } else if c == ']' {
                    kind = KindBuilder::try_from(builder.drain(..).collect::<String>().as_str())?;
                    kind_builder = false;
                } else {
                    builder.push(c);
                }
                continue;
            }
            if c == '|' {
                seperator = Some('|');
            } else if c == '&' {
                seperator = Some('&');
            } else {
                seperator = None;
            }
            if seperator.is_some() {
                let str = builder.drain(..).collect::<String>();
                let kinds = str
                    .split("<-")
                    .map(|s| Kind::new(kind.clone(), s.to_owned()))
                    .collect::<Result<Vec<_>, _>>()?;
                items.push((kinds, seperator == Some('|')));
            } else {
                if builder.is_empty() && c == '!' {
                    kind_builder = true
                } else if builder.is_empty() && c == ' ' {
                } else {
                    builder.push(c);
                }
            }
        }

        if !builder.is_empty() {
            if kind_builder {
                todo!("failed to close kind builder");
            }
            let kinds = builder
                .into_iter()
                .collect::<String>()
                .split("<-")
                .map(|s| Kind::new(kind.clone(), s.to_owned()))
                .collect::<Result<Vec<_>, _>>()?;
            items.push((kinds, false));
        }
        let name = name.into_iter().collect::<String>();

        Ok(Self {
            name: match name.trim().is_empty() {
                false => Some(name.trim().to_owned()),
                true => None,
            },
            queries: items,
            r_site: right,
        })
    }
}

#[derive(Debug)]
pub enum Kind {
    Text(Selector, bool),
    StripText(Selector, bool),
    Regex(Regex, bool),
    Html(Selector, bool),
    Attr(Selector, String, bool),
}

impl Kind {
    pub fn array(&self) -> bool {
        match self {
            Self::Text(_, b) => *b,
            Self::StripText(_, b) => *b,
            Self::Regex(_, b) => *b,
            Self::Html(_, b) => *b,
            Self::Attr(_, _, b) => *b,
        }
    }

    fn new(kind: KindBuilder, s: String) -> crate::Result<Self> {
        Ok(match kind {
            KindBuilder::Attr(attr, b) => Self::Attr(Selector::parse(&s)?, attr, b),
            KindBuilder::Text(b) => Self::Text(Selector::parse(&s)?, b),
            KindBuilder::StripText(b) => Self::StripText(Selector::parse(&s)?, b),
            KindBuilder::Regex(b) => Self::Regex(Regex::new(&s.trim())?, b),
            KindBuilder::Html(b) => Self::Html(Selector::parse(&s)?, b),
        })
    }
}

#[derive(Debug, Clone)]
pub enum KindBuilder {
    Text(bool),
    Attr(String, bool),
    StripText(bool),
    Regex(bool),
    Html(bool),
}

impl TryFrom<&str> for KindBuilder {
    type Error = InitError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.starts_with("@attr=") {
            let (_, s) = s.split_once("=").ok_or(InitError::InvalidAttrFormat)?;
            return Ok(Self::Attr(s.to_owned(), false));
        } else if s.starts_with("attr=") {
            let (_, s) = s.split_once("=").ok_or(InitError::InvalidAttrFormat)?;
            return Ok(Self::Attr(s.to_owned(), false));
        }
        match s {
            "text" => Ok(Self::Text(false)),
            "src" => Ok(Self::Attr("src".to_owned(), false)),
            "@src" => Ok(Self::Attr("src".to_owned(), true)),
            "href" => Ok(Self::Attr("href".to_owned(), false)),
            "@href" => Ok(Self::Attr("href".to_owned(), true)),
            "strip_text" => Ok(Self::StripText(false)),
            "regex" => Ok(Self::Regex(false)),
            "" | "html" => Ok(Self::Html(false)),
            "@text" => Ok(Self::Text(true)),
            "@strip_text" => Ok(Self::StripText(true)),
            "@regex" => Ok(Self::Regex(true)),
            "@" | "@html" => Ok(Self::Html(true)),
            _ => Err(InitError::UnknownAttr),
        }
    }
}

#[derive(Debug)]
pub struct MySelector {
    pub query: Query,
    pub children: Vec<MySelector>,
}

impl MySelector {
    pub fn parse(line: &str) -> crate::Result<(usize, MySelector)> {
        let (line, spaces) = strip_spaces_and_count(line);

        let query = Query::try_from(line.as_str())?;

        Ok((
            spaces + 1,
            MySelector {
                query,
                children: vec![],
            },
        ))
    }
}

fn strip_spaces_and_count(s: &str) -> (String, usize) {
    let mut count = 0;
    let mut s = s;
    while let Some(v) = s.strip_prefix(" ") {
        count += 1;
        s = v;
    }
    (s.to_owned(), count)
}

pub fn parse(s: &str) -> crate::Result<Vec<MySelector>> {
    let mut lines = s.lines().filter(|v| !v.trim().is_empty());
    let mut default = MySelector {
        query: Query {
            name: None,
            queries: vec![],
            r_site: None,
        },
        children: vec![],
    };
    while let Some(_) = parse_(&mut default, 0, &mut lines)? {}
    Ok(default.children)
}

fn parse_<'a>(
    parent: &mut MySelector,
    pindent: usize,
    lines: &mut impl Iterator<Item = &'a str>,
) -> crate::Result<Option<(usize, MySelector)>> {
    while let Some(line) = lines.next() {
        let (mut indent, mut tag) = MySelector::parse(line)?;
        loop {
            if indent > pindent {
                parent.children.push(tag);
                let v = parse_(parent.children.last_mut().unwrap(), indent, lines)?;
                match v {
                    Some((i, t)) => {
                        indent = i;
                        tag = t;
                    }
                    None => return Ok(None),
                }
            } else {
                return Ok(Some((indent, tag)));
            }
        }
    }
    Ok(None)
}
