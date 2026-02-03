use serde_json::Value;

use crate::processors::PostScraperProcessor;

#[derive(Default)]
pub struct JsonFlattenPostProcessor;
impl PostScraperProcessor for JsonFlattenPostProcessor {
    fn name(&self) -> &str {
        "json_flatten"
    }

    fn process(&self, data: Vec<String>, _: &str) -> Vec<String> {
        let mut out = vec![];
        for data in data {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                match v {
                    Value::Array(s) => {
                        out.extend(s.into_iter().map(|v| serde_json::to_string(&v).unwrap()))
                    }
                    _ => out.push(data),
                };
            } else {
                out.push(data);
            }
        }
        out
    }
}
