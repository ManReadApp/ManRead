use regex::{Captures, Regex};
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Field {
    pub name: String,
    target: Vec<(Target, Sel)>,
}

#[derive(Debug)]
enum Sel {
    Sel(Selector),
    Re(Regex),
}

impl Sel {
    fn as_regex(&self) -> &Regex {
        match self {
            Sel::Sel(_) => unreachable!(),
            Sel::Re(re) => &re,
        }
    }
    fn as_sel(&self) -> &Selector {
        match self {
            Sel::Sel(sel) => &sel,
            Sel::Re(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum Target {
    Html(Prefix),
    InnerHtml(Prefix),
    HtmlFn {
        prefix: Prefix,
        left: Box<Field>,
        right: Box<Field>,
    },
    Text(Prefix),
    StripText(Prefix),
    Regex(Prefix),
    Attr(Prefix, String),
}

#[derive(Debug)]
enum Prefix {
    None,
    All,
    Num(usize),
}

impl TryFrom<(&str, Option<&str>)> for Target {
    type Error = ();

    fn try_from((value, selector): (&str, Option<&str>)) -> Result<Self, Self::Error> {
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
            "inner_html" => Ok(Self::InnerHtml(pre)),
            "html_fn" => {
                let (l, r): (String, String) =
                    serde_json::from_str(selector.ok_or(())?).map_err(|_| ())?;
                let re = Field::regex();
                let l = re
                    .captures(&format!("left{}", l))
                    .map(|v| Field::parse_inner(v))
                    .ok_or(())?;
                let r = re
                    .captures(&format!("right{}", r))
                    .map(|v| Field::parse_inner(v))
                    .ok_or(())?;
                Ok(Self::HtmlFn {
                    prefix: pre,
                    left: Box::new(l),
                    right: Box::new(r),
                })
            }
            "text" => Ok(Self::Text(pre)),
            "strip_text" => Ok(Self::StripText(pre)),
            "regex" => Ok(Self::Regex(pre)),
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
        let re = Self::regex();
        let mut res = vec![];
        for cap in re.captures_iter(text) {
            res.push(Self::parse_inner(cap));
        }
        res
    }
    fn regex() -> Regex {
        Regex::new(r#"\b([a-zA-Z0-9_]+)\[([a-zA-Z0-9@_=\-]+)]\s(.+)"#)
            .expect("static regex pattern")
    }

    fn parse_inner(cap: Captures) -> Field {
        let (p1, p2) = match cap[3].split_once(" => ") {
            None => (&cap[3], None),
            Some((a, b)) => (a, Some(b)),
        };
        let target = format!("[{}] {}", &cap[2], p1)
            .split("|")
            .map(|new| {
                let hay = format!("name{}", new.trim());
                let item = Self::regex().captures(&hay).unwrap();
                (item[2].to_string(), item[3].to_string())
            })
            .map(|(target, selector)| {
                if let Ok(v) = Target::try_from((target.as_str(), p2)) {
                    let sel = match &v {
                        Target::Regex(_) => Sel::Re(Regex::new(&selector).unwrap()),
                        _ => Sel::Sel(Selector::parse(&selector).unwrap()),
                    };
                    (v, sel)
                } else {
                    panic!("Invalid target: {}", &target)
                }
            })
            .collect::<Vec<_>>();
        Field {
            name: cap[1].to_string(),
            target,
        }
    }

    fn get_single(&self, doc: &Html, target: &Target, selector: &Selector) -> Option<String> {
        let mut select = doc.select(&selector);
        Some(match target {
            Target::HtmlFn {
                prefix,
                left,
                right,
            } => match prefix {
                Prefix::None => {
                    let html = select.next()?.html();
                    serde_json::to_string(&vec![(left.get(&html), right.get(&html))]).ok()?
                }
                Prefix::All => {
                    let htmls = select
                        .map(|v| v.html())
                        .map(|html| match (left.get(&html), right.get(&html)) {
                            (Some(l), Some(r)) => Some((l, r)),
                            _ => None,
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    let htmls = v[..*size]
                        .iter()
                        .map(|v| v.html())
                        .map(|html| match (left.get(&html), right.get(&html)) {
                            (Some(l), Some(r)) => Some((l, r)),
                            _ => None,
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
            },
            Target::Html(prefix) => match prefix {
                Prefix::None => select.next()?.html(),
                Prefix::All => {
                    let htmls = select.map(|v| v.html()).collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    let htmls = v[..*size].iter().map(|v| v.html()).collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
            },
            Target::InnerHtml(prefix) => match prefix {
                Prefix::None => select.next()?.inner_html(),
                Prefix::All => {
                    let htmls = select.map(|v| v.inner_html()).collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    let htmls = v[..*size]
                        .iter()
                        .map(|v| v.inner_html())
                        .collect::<Vec<_>>();
                    if htmls.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&htmls).ok()?
                }
            },
            Target::Text(prefix) => match prefix {
                Prefix::None => get_text(select.next()?.text()),
                Prefix::All => {
                    let texts = select.map(|text| get_text(text.text())).collect::<Vec<_>>();
                    if texts.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&texts).ok()?
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    let texts = v[..*size]
                        .iter()
                        .map(|v| get_text(v.text()))
                        .collect::<Vec<_>>();
                    if texts.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&texts).ok()?
                }
            },
            Target::StripText(prefix) => match prefix {
                Prefix::None => clean_text(get_text(select.next()?.text()))
                    .trim()
                    .to_string(),
                Prefix::All => {
                    let texts = select
                        .map(|text| clean_text(get_text(text.text())).trim().to_string())
                        .collect::<Vec<_>>();
                    if texts.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&texts).ok()?
                }
                Prefix::Num(size) => {
                    let v = select.collect::<Vec<_>>();
                    let texts = v[..*size]
                        .iter()
                        .map(|v| clean_text(get_text(v.text())).trim().to_string())
                        .collect::<Vec<_>>();
                    if texts.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&texts).ok()?
                }
            },
            Target::Attr(prefix, v) => match prefix {
                Prefix::None => select.next()?.attr(v).unwrap_or_default().to_string(),
                Prefix::All => {
                    let v = select
                        .map(|refr| refr.attr(v).unwrap_or_default().to_string())
                        .collect::<Vec<_>>();
                    if v.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&v).ok()?
                }
                Prefix::Num(size) => {
                    let items = select.collect::<Vec<_>>();
                    let v = items[..*size]
                        .iter()
                        .map(|refr| refr.attr(v).unwrap_or_default().to_string())
                        .collect::<Vec<_>>();
                    if v.is_empty() {
                        return None;
                    }
                    serde_json::to_string(&v).ok()?
                }
            },
            Target::Regex(_) => {
                unreachable!()
            }
        })
    }

    pub fn get(&self, html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        for (target, selector) in &self.target {
            let temp = if let Target::Regex(pre) = target {
                let re = selector.as_regex();
                match pre {
                    Prefix::None => {
                        let data = &re.captures(html)?[1];
                        Some(data.to_string())
                    }
                    Prefix::All => {
                        let items: Vec<_> =
                            re.captures_iter(html).map(|v| v[1].to_string()).collect();
                        Some(serde_json::to_string(&items).ok()?)
                    }
                    Prefix::Num(num) => {
                        let items: Vec<_> =
                            re.captures_iter(html).map(|v| v[1].to_string()).collect();
                        let len = items.len();
                        let num = (*num).min(len);
                        Some(serde_json::to_string(&items[..num]).ok()?)
                    }
                }
            } else {
                self.get_single(&doc, target, selector.as_sel())
            };
            if let Some(v) = temp {
                return Some(v);
            }
        }
        None
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
