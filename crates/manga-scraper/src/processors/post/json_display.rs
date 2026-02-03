use serde_json::Value;

use crate::processors::PostScraperProcessor;

#[derive(Default)]
pub struct JsonDispayProcessor;
impl PostScraperProcessor for JsonDispayProcessor {
    fn name(&self) -> &str {
        "json_display"
    }

    fn process(&self, data: Vec<String>, _: &str) -> Vec<String> {
        let mut out = vec![];
        for data in data {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                out.push(match v {
                    Value::Null => "NULL".to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(number) => number.to_string(),
                    Value::String(s) => s,
                    v => serde_json::to_string(&v).unwrap(),
                });
            } else {
                out.push(data);
            }
        }
        out
    }
}
