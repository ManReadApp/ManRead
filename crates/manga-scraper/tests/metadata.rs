use scraper_testing::generate_tests;
use std::path::Path;

use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    Equal,
    StartsWith,
    EndsWith,
    Contains,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Len,
    Sorted,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub key: String,
    pub index: Option<usize>,
    pub operation: Option<Operation>,
    pub comparison: Comparison,
    pub expected: Value,
}

impl Rule {
    pub fn parse(rule_str: &str) -> Result<Self, String> {
        // Split into left and right parts based on comparison operator
        let (left_part, right_part, comparison) = split_line(rule_str)?;

        // Parse the left part (key, index, operation)
        let (key, index, operation) = parse_left_part(left_part)?;

        // Parse the right part (expected value)
        let expected = parse_right_part(right_part)?;

        Ok(Rule {
            key,
            index,
            operation,
            comparison,
            expected,
        })
    }
}

fn split_line(line: &str) -> Result<(&str, &str, Comparison), String> {
    let operators = [
        (" == ", Comparison::Equal),
        (" starts_with ", Comparison::StartsWith),
        (" ends_with ", Comparison::EndsWith),
        (" contains ", Comparison::Contains),
    ];

    let mut earliest_pos = None;
    let mut selected_op = None;

    for (op_str, comparison) in &operators {
        if let Some(pos) = line.find(op_str) {
            if earliest_pos.map_or(true, |ep| pos < ep) {
                earliest_pos = Some(pos);
                selected_op = Some((op_str, comparison));
            }
        }
    }

    let (op_str, comparison) =
        selected_op.ok_or_else(|| "No valid comparison operator found".to_string())?;
    let pos = earliest_pos.unwrap();

    let left_part = line[..pos].trim();
    let right_part = line[pos + op_str.len()..].trim();

    Ok((left_part, right_part, *comparison))
}

fn parse_left_part(left_part: &str) -> Result<(String, Option<usize>, Option<Operation>), String> {
    let tokens: Vec<&str> = left_part.split_whitespace().collect();

    // Check for operation suffix
    let (operation, key_tokens) = match tokens.as_slice() {
        [.., "len"] => (Some(Operation::Len), &tokens[..tokens.len() - 1]),
        [.., "sorted"] => (Some(Operation::Sorted), &tokens[..tokens.len() - 1]),
        _ => (None, &tokens[..]),
    };

    let key_str = key_tokens.join(" ");
    let (key_base, index) = parse_key(&key_str)?;

    Ok((key_base, index, operation))
}

fn parse_key(key_str: &str) -> Result<(String, Option<usize>), String> {
    if let Some(bracket_start) = key_str.rfind('[') {
        if !key_str.ends_with(']') {
            return Err("Missing closing bracket in key".to_string());
        }

        let index_part = &key_str[bracket_start + 1..key_str.len() - 1];
        let index = index_part
            .parse::<usize>()
            .map_err(|_| format!("Invalid index: {}", index_part))?;

        let key_base = key_str[..bracket_start].trim().to_string();
        return Ok((key_base, Some(index)));
    }

    Ok((key_str.to_string(), None))
}

fn parse_right_part(right_part: &str) -> Result<Value, String> {
    serde_json::from_str(right_part).map_err(|e| format!("Failed to parse expected value: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_rule() {
        let rule = Rule::parse(r#"title == ["Revenge of the Baskerville Bloodhound"]"#).unwrap();
        assert_eq!(rule.key, "title");
        assert_eq!(rule.index, None);
        assert_eq!(rule.operation, None);
        assert_eq!(rule.comparison, Comparison::Equal);
        assert_eq!(
            rule.expected,
            json!(["Revenge of the Baskerville Bloodhound"])
        );
    }

    #[test]
    fn test_parse_with_index_and_operation() {
        let rule = Rule::parse(r#"img[0] starts_with "https://cdn.anime-planet.com""#).unwrap();
        assert_eq!(rule.key, "img");
        assert_eq!(rule.index, Some(0));
        assert_eq!(rule.operation, None);
        assert_eq!(rule.comparison, Comparison::StartsWith);
        assert_eq!(rule.expected, json!("https://cdn.anime-planet.com"));
    }

    #[test]
    fn test_parse_with_len_operation() {
        let rule = Rule::parse("tags len == 13").unwrap();
        assert_eq!(rule.key, "tags");
        assert_eq!(rule.index, None);
        assert_eq!(rule.operation, Some(Operation::Len));
        assert_eq!(rule.comparison, Comparison::Equal);
        assert_eq!(rule.expected, json!(13));
    }

    #[test]
    fn test_parse_with_sorted_operation() {
        let rule = Rule::parse(r#"tags sorted == ["Action", "Fantasy"]"#).unwrap();
        assert_eq!(rule.key, "tags");
        assert_eq!(rule.index, None);
        assert_eq!(rule.operation, Some(Operation::Sorted));
        assert_eq!(rule.comparison, Comparison::Equal);
        assert_eq!(rule.expected, json!(["Action", "Fantasy"]));
    }

    #[test]
    fn test_parse_invalid_rule() {
        let result = Rule::parse("invalid key[");
        assert!(result.is_err());
    }
}

async fn test_logic(_: &str, content: &str) {
    let items = manga_scraper::init::register(Path::new("../../data/")).unwrap();
    let queries = content.split("\n\n");
    for query in queries {
        let mut query = query.lines();
        let url = query.next().unwrap().trim();
        let query = query.collect::<Vec<_>>();
        if url.is_empty() {
            continue;
        }
        let service = items.get(url).unwrap();
        let meta = service
            .metadata
            .as_ref()
            .unwrap()
            .scrape_metadata(url)
            .await
            .unwrap();
        println!("{:#?}", meta);
        if query.len() == 0 {
            for (key, value) in meta {
                println!(
                    "{key} == {}",
                    serde_json::to_string(&Value::from(value)).unwrap()
                );
            }
            panic!("No validation for item");
        }
        for query in query {
            let rule = Rule::parse(query).expect(&query);
            let value = meta
                .get(&rule.key)
                .expect(format!("missing key {}", rule.key).as_str());
            let value = if let Some(index) = rule.index {
                match value {
                    scraper_module::ScrapedData::Arr(scraped_datas) => {
                        scraped_datas.get(index).expect("Out of range")
                    }
                    data => panic!("tried to get index from {:?}", data),
                }
            } else {
                value
            };
            let value = if let Some(operation) = rule.operation {
                match operation {
                    Operation::Len => json!(match value {
                        scraper_module::ScrapedData::Str(s) => s.len(),
                        scraper_module::ScrapedData::Arr(scraped_datas) => scraped_datas.len(),
                        scraper_module::ScrapedData::Map(hash_map) => hash_map.len(),
                        scraper_module::ScrapedData::Map2(hash_map) => hash_map.len(),
                    }),
                    Operation::Sorted => match value {
                        scraper_module::ScrapedData::Arr(scraped_datas) => {
                            let mut scraped_datas = scraped_datas.clone();
                            scraped_datas.sort_by(|a, b| a.cmp(b));
                            Value::Array(
                                scraped_datas.into_iter().map(|v| Value::from(v)).collect(),
                            )
                        }
                        _ => panic!("tried to sort {:?}", value),
                    },
                }
            } else {
                Value::from(value.clone())
            };
            match rule.comparison {
                Comparison::Equal => assert_eq!(value, rule.expected),
                Comparison::StartsWith => {
                    if let (Value::String(value_str), Value::String(expected_str)) =
                        (&value, &rule.expected)
                    {
                        assert!(
                            value_str.starts_with(expected_str),
                            "Value '{}' does not start with '{}'",
                            value_str,
                            expected_str
                        );
                    } else {
                        panic!("Expected both values to be strings for StartsWith comparison. {:?} {:?}",
                        value, rule.expected
                    );
                    }
                }

                Comparison::EndsWith => {
                    if let (Value::String(value_str), Value::String(expected_str)) =
                        (&value, &rule.expected)
                    {
                        assert!(
                            value_str.ends_with(expected_str),
                            "Value '{}' does not end with '{}'",
                            value_str,
                            expected_str
                        );
                    } else {
                        panic!(
                            "Expected both values to be strings for EndsWith comparison.{:?} {:?}",
                            value, rule.expected
                        );
                    }
                }

                Comparison::Contains => {
                    if let (Value::String(value_str), Value::String(expected_str)) =
                        (&value, &rule.expected)
                    {
                        assert!(
                            value_str.contains(expected_str),
                            "Value '{}' does not contain '{}'",
                            value_str,
                            expected_str
                        );
                    } else {
                        panic!(
                            "Expected both values to be strings for Contains comparison. {:?} {:?}",
                            value, rule.expected
                        );
                    }
                }
            }
        }
    }
}

// Generates tokio::test with sturcture uri_test for every .metadata_test file
// Execute specific test with:
// cargo test -p manga-scraper anime_planet_metadata_test -- --nocapture
generate_tests!((test_logic, "data/external", "metadata_test"));
