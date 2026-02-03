use serde_json::Value;

use crate::processors::PostScraperProcessor;

#[derive(Default)]
pub struct JsonStringStringPostProcessor;
impl PostScraperProcessor for JsonStringStringPostProcessor {
    fn name(&self) -> &str {
        "json_string_string"
    }

    fn process(&self, data: Vec<String>, _: &str) -> Vec<String> {
        let mut out = vec![];
        for data in data {
            if let Ok(v) = serde_json::from_str::<Value>(&data) {
                match v {
                    Value::String(s) => match serde_json::from_str::<Value>(&s) {
                        Ok(v) => out.push(serde_json::to_string(&v).unwrap()),
                        Err(_) => out.push(data),
                    },
                    _ => out.push(data),
                };
            } else {
                out.push(data);
            }
        }
        out
    }
}
