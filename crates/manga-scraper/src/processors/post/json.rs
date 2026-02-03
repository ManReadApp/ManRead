use serde_json::Value;

use crate::processors::PostScraperProcessor;

fn get_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        if let Some((key, idx)) = part.strip_suffix(']').and_then(|p| {
            let mut split = p.split('[');
            Some((split.next()?, split.next()?.parse::<usize>().ok()?))
        }) {
            current = current.get(key)?.get(idx)?;
        } else {
            current = current.get(part)?;
        }
    }
    Some(current)
}
#[derive(Default)]
pub struct JsonPostProcessor;
impl PostScraperProcessor for JsonPostProcessor {
    fn name(&self) -> &str {
        "json"
    }

    fn process(&self, data: Vec<String>, target: &str) -> Vec<String> {
        let mut out = vec![];
        for data in data {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                if let Some(v) = get_by_path(&v, target) {
                    out.push(serde_json::to_string(v).unwrap());
                } else {
                    out.push(data);
                }
            } else {
                out.push(data);
            }
        }
        out
    }
}
