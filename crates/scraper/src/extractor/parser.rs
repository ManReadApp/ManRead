use regex::Regex;
use scraper::{Html, Selector};

pub struct Field {
    pub name: String,
    target: Target,
    selector: Selector,
}

enum Target {
    Html(Prefix),
    Text(Prefix),
    StripText(Prefix),
    Attr(Prefix, String),
}

#[derive(Debug)]
enum Prefix {
    None,
    All,
    Num(usize),
}

impl TryFrom<&str> for Target {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value = value.to_lowercase();
        let prefix = value.chars().next();
        let mut pre = Prefix::None;
        // Check if the prefix is '@', a digit, or None
        if let Some('@') | Some('0'..='9') = prefix {
            // If so, remove the prefix
            if let Some(v) = value.strip_prefix('@') {
                pre = Prefix::All;
                value = v.to_string();
            } else if let Some(index) = value.find(|c: char| !c.is_ascii_digit()) {
                let numeric_part = &value[..index];
                if let Ok(num) = numeric_part.parse::<usize>() {
                    pre = Prefix::Num(num);
                    value = value[index..].to_owned();
                }
            }
        }
        match value.as_str() {
            "html" => Ok(Self::Html(pre)),
            "text" => Ok(Self::Text(pre)),
            "strip_text" => Ok(Self::StripText(pre)),
            "src" => Ok(Self::Attr(pre, "src".to_string())),
            "href" => Ok(Self::Attr(pre, "href".to_string())),
            _ => {
                if let Some(v) = value.strip_prefix("attr=") {
                    Ok(Self::Attr(pre, v.to_string()))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl Field {
    pub fn parse(text: &str) -> Vec<Field> {
        let re = Regex::new(r#"\b([a-zA-Z0-9_]+)\[([a-zA-Z0-9@_=\-]+)]\s(.+)"#).unwrap();
        let mut res = vec![];
        for cap in re.captures_iter(text) {
            if let Ok(v) = Target::try_from(&cap[2]) {
                res.push(Field {
                    name: cap[1].to_string(),
                    target: v,
                    selector: Selector::parse(&cap[3]).unwrap(),
                });
            } else {
                panic!("Invalid target: {}", &cap[2])
            }
        }
        res
    }

    pub fn get(&self, html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let mut select = doc.select(&self.selector);
        Some(match &self.target {
            Target::Html(prefix) => match prefix {
                Prefix::None => select.next()?.html(),
                Prefix::All => {
                    serde_json::to_string(&select.map(|v| v.html()).collect::<Vec<_>>()).unwrap()
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    serde_json::to_string(&v[..*size].iter().map(|v| v.html()).collect::<Vec<_>>())
                        .unwrap()
                }
            },
            Target::Text(prefix) => match prefix {
                Prefix::None => get_text(select.next()?.text()),
                Prefix::All => serde_json::to_string(
                    &select.map(|text| get_text(text.text())).collect::<Vec<_>>(),
                )
                .unwrap(),
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    serde_json::to_string(
                        &v[..*size]
                            .iter()
                            .map(|v| get_text(v.text()))
                            .collect::<Vec<_>>(),
                    )
                    .unwrap()
                }
            },
            Target::StripText(prefix) => match prefix {
                Prefix::None => clean_text(get_text(select.next()?.text()))
                    .trim()
                    .to_string(),
                Prefix::All => serde_json::to_string(
                    &select
                        .map(|text| clean_text(get_text(text.text())).trim().to_string())
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    serde_json::to_string(
                        &v[..*size]
                            .iter()
                            .map(|v| clean_text(get_text(v.text())).trim().to_string())
                            .collect::<Vec<_>>(),
                    )
                    .unwrap()
                }
            },
            Target::Attr(prefix, v) => match prefix {
                Prefix::None => select.next()?.attr(v).unwrap_or_default().to_string(),
                Prefix::All => serde_json::to_string(
                    &select
                        .map(|refr| refr.attr(v).unwrap_or_default().to_string())
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
                Prefix::Num(size) => {
                    let items = select.collect::<Vec<_>>();
                    serde_json::to_string(
                        &items[..*size]
                            .iter()
                            .map(|refr| refr.attr(v).unwrap_or_default().to_string())
                            .collect::<Vec<_>>(),
                    )
                    .unwrap()
                }
            },
        })
    }
}

fn get_text(text: scraper::element_ref::Text) -> String {
    text.collect()
}
pub fn clean_text(text: String) -> String {
    if let Some(v) = text.strip_prefix('\n') {
        clean_text(v.to_string())
    } else if let Some(v) = text.strip_suffix('\n') {
        clean_text(v.to_string())
    } else if let Some(v) = text.strip_prefix(' ') {
        clean_text(v.to_string())
    } else if let Some(v) = text.strip_suffix(' ') {
        clean_text(v.to_string())
    } else {
        text
    }
}
