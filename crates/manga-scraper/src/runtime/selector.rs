use std::collections::{BTreeMap, HashMap};

use scraper::Element;
use scraper::{ElementRef, Html};
use scraper_module::{ScrapedData, ScraperError};

use crate::init::parse::selectors::{Kind, KindBuilder, MySelector, Query};

#[derive(Debug)]
pub enum OutData {
    String(String),
    Array(Vec<OutData>),
    Tuple((Option<String>, Box<OutData>)),
}

impl OutData {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            OutData::String(s) => Some(s.as_str()),
            OutData::Array(v) => match v.len() == 1 {
                true => v.get(0).unwrap().as_str(),
                false => None,
            },
            _ => None,
        }
    }
}

fn el_to_str(data: &ElementOrText<'_>) -> String {
    match data {
        ElementOrText::Element(element_ref) => element_ref.text().collect::<String>(),
        ElementOrText::Text(t) => t.to_owned(),
    }
}

pub fn flatten(data: OutData) -> ScrapedData {
    match data {
        OutData::String(value) => ScrapedData::Str(value),
        OutData::Array(items) => {
            let v = items.into_iter().map(flatten).collect::<Vec<_>>();
            let all_arrays = v
                .iter()
                .all(|item| matches!(item, ScrapedData::Arr(_) | ScrapedData::Str(_)));

            if all_arrays {
                let temp = ScrapedData::Arr(
                    v.into_iter()
                        .flat_map(|item| match item {
                            ScrapedData::Str(inner) => vec![ScrapedData::Str(inner)],
                            ScrapedData::Arr(inner) => inner,
                            _ => unreachable!(),
                        })
                        .collect(),
                );

                return temp;
            }

            let all_maps = v.iter().all(|item| matches!(item, ScrapedData::Map(_)));
            if all_maps {
                let data = v
                    .into_iter()
                    .map(|item| match item {
                        ScrapedData::Map(map) => map,
                        _ => unreachable!(),
                    })
                    .fold(HashMap::new(), |mut acc, map| {
                        for (key, value) in map {
                            if let Some(value) = value.as_array() {
                                acc.entry(key).or_insert_with(Vec::new).extend(value);
                            } else {
                                acc.entry(key).or_insert_with(Vec::new).push(value);
                            }
                        }
                        acc
                    });
                return ScrapedData::Map(
                    data.into_iter()
                        .map(|(key, values)| (key, ScrapedData::Arr(values)))
                        .collect(),
                );
            }

            todo!("throw error for mixed types or unsupported structures")
        }
        OutData::Tuple((opt_key, value)) => match opt_key {
            None => ScrapedData::Arr(vec![flatten(*value)]),
            Some(key) => ScrapedData::Map(BTreeMap::from([(key, flatten(*value))])),
        },
    }
}

impl KindBuilder {
    pub fn to_data(&self, data: Vec<ElementOrText<'_>>) -> Result<OutData, ScraperError> {
        Ok(match self {
            Self::Attr(attr, v) => match *v {
                true => data
                    .first()
                    .ok_or(ScraperError::NodeNotFound)
                    .and_then(|v| {
                        Ok(OutData::String(
                            v.attr(attr).ok_or(ScraperError::AttrNotFound)?.to_owned(),
                        ))
                    })?,
                false => OutData::Array(
                    data.iter()
                        .map(|v| {
                            Ok(OutData::String(
                                v.attr(attr).ok_or(ScraperError::AttrNotFound)?.to_owned(),
                            ))
                        })
                        .collect::<Result<Vec<_>, ScraperError>>()?,
                ),
            },
            Self::Text(v) => match *v {
                true => data
                    .first()
                    .map(|v| OutData::String(el_to_str(v)))
                    .ok_or(ScraperError::NodeNotFound)?,
                false => OutData::Array(
                    data.iter()
                        .map(el_to_str)
                        .map(OutData::String)
                        .collect::<Vec<_>>(),
                ),
            },
            Self::StripText(v) => match *v {
                true => data
                    .first()
                    .map(|v| OutData::String(el_to_str(v).trim().to_owned()))
                    .ok_or(ScraperError::NodeNotFound)?,
                false => OutData::Array(
                    data.iter()
                        .map(el_to_str)
                        .map(|v| v.trim().to_owned())
                        .map(OutData::String)
                        .collect::<Vec<_>>(),
                ),
            },
            Self::Regex(v) => match *v {
                true => data
                    .first()
                    .map(|v| OutData::String(el_to_str(v)))
                    .ok_or(ScraperError::NodeNotFound)?,
                false => OutData::Array(
                    data.iter()
                        .map(el_to_str)
                        .map(OutData::String)
                        .collect::<Vec<_>>(),
                ),
            },
            Self::Html(v) => match *v {
                true => data
                    .first()
                    .map(|v| OutData::String(v.html()))
                    .ok_or(ScraperError::NodeNotFound)?,
                false => OutData::Array(
                    data.iter()
                        .map(|v| v.html())
                        .map(OutData::String)
                        .collect::<Vec<_>>(),
                ),
            },
        })
    }
}

impl MySelector {
    pub fn run(&self, data: &str) -> Result<(Option<String>, OutData), ScraperError> {
        let document = Html::parse_document(data);
        let document = SelectorEnum::Document(&document);
        self.run_with_doc(document)
    }

    pub fn run_with_doc(
        &self,
        document: SelectorEnum,
    ) -> Result<(Option<String>, OutData), ScraperError> {
        let data = self.query.run_with_doc(document);
        if !self.children.is_empty() {
            if data.r.is_some() {
                return Err(ScraperError::Unimplemented);
            }
            let mut out = vec![];
            for elem in data.data {
                let l = SelectorEnum::Ref(vec![elem]);
                let data = self
                    .children
                    .iter()
                    .map(|v| {
                        v.run_with_doc(l.clone())
                            .map(|v| (v.0, Box::new(v.1)))
                            .map(OutData::Tuple)
                    })
                    .collect::<Result<Vec<_>, ScraperError>>()?;
                out.push(OutData::Array(data));
            }

            Ok((self.query.name.clone(), OutData::Array(out)))
        } else {
            join_if_possible(data, self.query.name.clone())
        }
    }
}

fn join_if_possible(
    data: DataLR,
    query: Option<String>,
) -> Result<(Option<String>, OutData), ScraperError> {
    if let Some(d) = data.r {
        let (_, right) = join_if_possible(*d, None)?;
        let mut data = data
            .data
            .into_iter()
            .map(|(elem, kind)| kind.to_data(vec![elem]))
            .collect::<Result<Vec<_>, _>>()?;
        if data.len() == 1 {
            let f = data.remove(0);
            if let Some(v) = f.as_str() {
                Ok((Some(v.to_string()), right))
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    } else {
        Ok((
            query,
            OutData::Array(
                data.data
                    .into_iter()
                    .map(|(elem, kind)| kind.to_data(vec![elem]))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ))
    }
}

pub struct DataLR<'a> {
    pub data: Vec<(ElementOrText<'a>, KindBuilder)>,
    r: Option<Box<DataLR<'a>>>,
}

impl Kind {
    pub fn select(&self) {}
}

#[derive(Clone)]
pub enum SelectorEnum<'a> {
    Document(&'a Html),
    Ref(Vec<(ElementOrText<'a>, KindBuilder)>),
}

#[derive(Clone)]
pub enum ElementOrText<'a> {
    Element(ElementRef<'a>),
    Text(String),
}

impl ElementOrText<'_> {
    pub fn html(&self) -> String {
        match self {
            ElementOrText::Element(element_ref) => element_ref.html(),
            ElementOrText::Text(s) => s.to_owned(),
        }
    }

    pub fn attr(&self, attr: &str) -> Option<&str> {
        match self {
            ElementOrText::Element(element_ref) => element_ref.attr(attr),
            ElementOrText::Text(_) => None,
        }
    }
}

impl<'a> SelectorEnum<'a> {
    pub fn len(&self) -> usize {
        match self {
            SelectorEnum::Document(_) => 0,
            SelectorEnum::Ref(vec) => vec.len(),
        }
    }
    pub fn get_refs(self) -> Vec<(ElementOrText<'a>, KindBuilder)> {
        match self {
            SelectorEnum::Document(_) => vec![],
            SelectorEnum::Ref(vec) => vec,
        }
    }
    pub fn extend(&mut self, parts: Vec<(ElementOrText<'a>, KindBuilder)>) {
        match self {
            SelectorEnum::Document(_) => *self = SelectorEnum::Ref(parts),
            SelectorEnum::Ref(vec) => {
                vec.extend(parts);
            }
        }
    }
    pub fn select(&mut self, selector: &Kind) {
        *self = SelectorEnum::Ref(match self {
            SelectorEnum::Document(document) => match selector {
                Kind::Text(selector, b) => document
                    .select(selector)
                    .map(|v| (ElementOrText::Element(v), KindBuilder::Text(*b)))
                    .collect(),
                Kind::StripText(selector, b) => document
                    .select(selector)
                    .map(|v| (ElementOrText::Element(v), KindBuilder::StripText(*b)))
                    .collect(),
                Kind::Regex(q, b) => q
                    .find_iter(&document.html())
                    .map(|v| {
                        (
                            ElementOrText::Text(v.as_str().to_owned()),
                            KindBuilder::Regex(*b),
                        )
                    })
                    .collect::<Vec<_>>(),
                Kind::Html(selector, b) => document
                    .select(selector)
                    .map(|v| (ElementOrText::Element(v), KindBuilder::Html(*b)))
                    .collect(),
                Kind::Attr(selector, attr, b) => document
                    .select(selector)
                    .map(|v| {
                        (
                            ElementOrText::Element(v),
                            KindBuilder::Attr(attr.to_owned(), *b),
                        )
                    })
                    .collect(),
            },
            SelectorEnum::Ref(refs) => refs
                .iter()
                .filter_map(|v| match &v.0 {
                    ElementOrText::Text(_) => None,
                    ElementOrText::Element(e) => Some((e, &v.1)),
                })
                .flat_map(|(v, _)| match selector {
                    Kind::Text(selector, b) => v
                        .select(selector)
                        .map(|v| (ElementOrText::Element(v), KindBuilder::Text(*b)))
                        .collect::<Vec<_>>(),
                    Kind::StripText(selector, b) => v
                        .select(selector)
                        .map(|v| (ElementOrText::Element(v), KindBuilder::StripText(*b)))
                        .collect::<Vec<_>>(),
                    Kind::Regex(q, b) => q
                        .find_iter(&v.html())
                        .map(|v| {
                            (
                                ElementOrText::Text(v.as_str().to_owned()),
                                KindBuilder::Regex(*b),
                            )
                        })
                        .collect::<Vec<_>>(),
                    Kind::Html(selector, b) => v
                        .select(selector)
                        .map(|v| (ElementOrText::Element(v), KindBuilder::Html(*b)))
                        .collect::<Vec<_>>(),
                    Kind::Attr(selector, attr, b) => v
                        .select(selector)
                        .map(|v| {
                            (
                                ElementOrText::Element(v),
                                KindBuilder::Attr(attr.to_owned(), *b),
                            )
                        })
                        .collect::<Vec<_>>(),
                })
                .collect(),
        });
    }
    pub fn parent(&mut self) {
        *self = SelectorEnum::Ref(match self {
            SelectorEnum::Document(_) => vec![],
            SelectorEnum::Ref(vec) => vec
                .into_iter()
                .filter_map(|v| match &v.0 {
                    ElementOrText::Text(_) => None,
                    ElementOrText::Element(e) => Some((e, &v.1)),
                })
                .filter_map(|(v, k)| {
                    v.parent_element()
                        .map(|v| (ElementOrText::Element(v), k.clone()))
                })
                .collect(),
        })
    }
}

impl Query {
    pub fn run_with_doc<'a>(&self, document: SelectorEnum<'a>) -> DataLR<'a> {
        let mut all = SelectorEnum::Ref(vec![]);
        for (index, (segments, or)) in self.queries.iter().enumerate() {
            let mut part = document.clone();
            for (index, selector) in segments.iter().enumerate() {
                if index != 0 {
                    part.parent();
                }
                part.select(selector);
            }

            if segments.first().map(|v| v.array()).unwrap() {
                all.extend(part.get_refs());
            } else {
                let mut refs = part.get_refs();
                if !refs.is_empty() {
                    all.extend(vec![refs.remove(0)]);
                }
            }
            if index + 1 != self.queries.len() {
                if *or && all.len() > 0 {
                    break;
                }
            }
        }

        let right = if let Some(v) = &self.r_site {
            Some(v.run_with_doc(document))
        } else {
            None
        };
        DataLR {
            data: all.get_refs(),
            r: right.map(Box::new),
        }
    }
}
