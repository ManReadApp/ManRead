use api_structure::models::manga::search::{Array, Field, Item, ItemData, ItemOrArray};

pub fn search_parser(s: &str, or_default: bool, field: &Vec<Field>) -> (Array, Vec<String>) {
    let mut depth = 0;
    let mut items: Vec<ItemOrArray> = vec![];
    let mut section: Vec<char> = vec![];
    let mut double_quote = false;
    let mut escape_next = false;
    let mut errors = vec![];
    let mut push_err = |v: Result<(), String>| {
        if let Err(err) = v {
            errors.push(err);
        }
    };
    for c in s.chars() {
        if escape_next {
            escape_next = false;
            section.push(c);
            continue;
        }
        if c == '\\' {
            escape_next = true;
            continue;
        }
        if c == '"' {
            double_quote = !double_quote;
            if !double_quote {
                section.push(c);
                push_err(push(
                    &mut items,
                    UnparsedItem::Item(section.drain(..).collect()),
                    depth,
                    0,
                    field,
                ));
                continue;
            }
        }
        if double_quote {
            section.push(c);
            continue;
        }

        if c == ' ' {
            push_err(push(
                &mut items,
                UnparsedItem::Item(section.drain(..).collect()),
                depth,
                0,
                field,
            ));
            continue;
        } else if c == '(' {
            depth += 1;
            if section == vec!['o', 'r', ':'] {
                push_err(push(&mut items, UnparsedItem::List(true), depth, 0, field));
                section.drain(..);
            } else if section == vec!['a', 'n', 'd', ':'] {
                push_err(push(&mut items, UnparsedItem::List(false), depth, 0, field));
                section.drain(..);
            } else {
                push_err(push(
                    &mut items,
                    UnparsedItem::List(or_default),
                    depth,
                    0,
                    field,
                ));
                section.drain(..);
            }
            continue;
        } else if c == ')' {
            push_err(push(
                &mut items,
                UnparsedItem::Item(section.drain(..).collect()),
                depth,
                0,
                field,
            ));
            depth = depth.saturating_sub(1);
            continue;
        }
        section.push(c);
    }
    push_err(push(
        &mut items,
        UnparsedItem::Item(section.drain(..).collect()),
        depth,
        0,
        field,
    ));
    (
        Array {
            or: or_default,
            items,
        },
        errors,
    )
}

#[derive(Debug)]
enum UnparsedItem {
    Item(String),
    List(bool),
}

fn push(
    arr: &mut Vec<ItemOrArray>,
    item: UnparsedItem,
    depth: usize,
    d_l: usize,
    field: &Vec<Field>,
) -> Result<(), String> {
    if let UnparsedItem::Item(s) = &item {
        if s.is_empty() || s == " " {
            return Ok(());
        }
    }

    if let Some(ItemOrArray::Array(v)) = arr.last_mut() {
        if d_l == depth {
            arr.push(try_from_str(item, field)?);
        } else {
            push(&mut v.items, item, depth, d_l + 1, field)?;
        }
    } else {
        arr.push(try_from_str(item, field)?);
    }

    Ok(())
}

fn try_from_str(s: UnparsedItem, fields: &[Field]) -> Result<ItemOrArray, String> {
    Ok(match s {
        UnparsedItem::Item(it) => {
            let not = it.contains(":!");
            let (category, mut search) = if not {
                let (c, s) = it.split_once(":!").unwrap();
                (c, s.to_string())
            } else {
                match it.split_once(':') {
                    Some((c, s)) => (c, s.to_string()),
                    None => ("title", it),
                }
            };
            if search.starts_with('"') && search.ends_with('"') && search.len() > 1 {
                search = search
                    .strip_prefix('"')
                    .unwrap()
                    .strip_suffix('"')
                    .unwrap()
                    .to_string();
            }
            let name = category.to_lowercase();
            let name = name.as_str();
            let field = fields
                .iter()
                .find(|v| v.name.to_lowercase().as_str() == name);
            if field.is_none() {
                return Err(format!("Category: {} not found", category));
            }
            let field = field.unwrap();
            let value = field.kind.parse(&search)?;
            let data = ItemData {
                name: field.name.to_string(),
                value,
            };
            ItemOrArray::Item(Item { not, data })
        }
        UnparsedItem::List(or) => ItemOrArray::Array(Array { or, items: vec![] }),
    })
}
