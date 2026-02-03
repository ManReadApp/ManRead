use serde_json::Value;

use crate::processors::PostScraperProcessor;

fn get_column_by_index(value: &Value, index: usize) -> Option<Value> {
    let data = value
        .as_array()?
        .iter()
        .map(|item| item.get(index))
        .map(|v| v.cloned())
        .collect::<Option<Vec<Value>>>()?;
    Some(Value::Array(data))
}

#[derive(Default)]
pub struct TakeJsonPostProcessor;
impl PostScraperProcessor for TakeJsonPostProcessor {
    fn name(&self) -> &str {
        "take_json"
    }

    fn process(&self, data: Vec<String>, target: &str) -> Vec<String> {
        let mut out = vec![];
        let target = match target.trim().parse::<usize>() {
            Ok(v) => v,
            Err(_) => return data,
        };
        for data in data {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                if let Some(v) = get_column_by_index(&v, target) {
                    out.push(serde_json::to_string(&v).unwrap());
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
