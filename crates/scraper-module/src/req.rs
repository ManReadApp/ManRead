use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

use crate::ScraperError;

pub trait ClientT: Send + Sync {
    fn method(&self, method: &str, url: &str) -> RequestBuilder;
    fn post(&self, url: &str) -> RequestBuilder {
        self.method("POST", url)
    }
    fn get(&self, url: &str) -> RequestBuilder {
        self.method("GET", url)
    }
    fn delete(&self, url: &str) -> RequestBuilder {
        self.method("DELETE", url)
    }
    fn put(&self, url: &str) -> RequestBuilder {
        self.method("PUT", url)
    }
    fn patch(&self, url: &str) -> RequestBuilder {
        self.method("PATCH", url)
    }
}

pub struct Response {
    pub r: Box<dyn ResponseT>,
}

impl Response {
    #[cfg(feature = "json")]
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, ScraperError> {
        return Ok(serde_json::from_slice(&self.bytes().await?)?);
    }

    pub async fn text(self) -> Result<String, ScraperError> {
        Ok(String::from_utf8(self.bytes().await?)?)
    }
    pub async fn bytes(self) -> Result<Vec<u8>, ScraperError> {
        self.r.bytes().await
    }

    pub fn headers(&self) -> Vec<(&str, &str)> {
        self.r.headers()
    }

    pub fn status(&self) -> u16 {
        self.r.status()
    }
}
pub struct RequestBuilder {
    pub r: Box<dyn RequestBuilderT>,
}

impl RequestBuilder {
    pub fn header(mut self, key: &str, value: impl ToString) -> RequestBuilder {
        self.r.set_header(key, value.to_string().as_str());
        self
    }

    pub fn headers(mut self, headers: HashMap<String, Vec<u8>>) -> RequestBuilder {
        self.r.set_headers(headers);
        self
    }

    pub fn body(mut self, body: impl ToString) -> RequestBuilder {
        self.r.set_body(body.to_string());
        self
    }

    pub fn basic_auth(mut self, username: &str, password: Option<&str>) -> RequestBuilder {
        self.r.set_basic_auth(username, password);
        self
    }

    pub fn bearer_auth(mut self, token: &str) -> RequestBuilder {
        self.r.set_bearer_auth(token);
        self
    }

    pub fn form(mut self, form: &HashMap<String, String>) -> RequestBuilder {
        self.r.set_form(form);
        self
    }

    pub fn json(mut self, json: MyValue) -> RequestBuilder {
        #[cfg(feature = "json")]
        self.r.set_json(json);
        self
    }

    pub async fn send(self) -> Result<Response, ScraperError> {
        self.r.send().await
    }
}
#[cfg(feature = "json")]
pub use serde_json::Value;

#[async_trait]
pub trait RequestBuilderT: Send + Sync {
    fn set_header(&mut self, key: &str, value: &str);

    fn set_headers(&mut self, headers: HashMap<String, Vec<u8>>);

    fn set_body(&mut self, body: String);

    fn set_basic_auth(&mut self, username: &str, password: Option<&str>);

    fn set_bearer_auth(&mut self, token: &str);

    fn set_form(&mut self, form: &HashMap<String, String>);

    async fn send(self: Box<Self>) -> Result<Response, ScraperError>;

    fn set_json(&mut self, json: MyValue);
}

#[derive(Clone)]
pub enum MyValue {
    /// Represents a JSON null value.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(null);
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(true);
    /// ```
    Bool(bool),

    /// Represents a JSON number, whether integer or floating point.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(12.5);
    /// ```
    Number(Number),

    /// Represents a JSON string.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!("a string");
    /// ```
    String(String),

    /// Represents a JSON array.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(["an", "array"]);
    /// ```
    Array(Vec<MyValue>),

    /// Represents a JSON object.
    ///
    /// By default the map is backed by a BTreeMap. Enable the `preserve_order`
    /// feature of serde_json to use IndexMap instead, which preserves
    /// entries in the order they are inserted into the map. In particular, this
    /// allows JSON data to be deserialized into a Value and serialized to a
    /// string while retaining the order of map keys in the input.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!({ "an": "object" });
    /// ```
    Object(BTreeMap<String, MyValue>),
}

#[derive(Copy, Clone)]
pub enum Number {
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
}

#[cfg(feature = "json")]
impl From<MyValue> for Value {
    fn from(value: MyValue) -> Self {
        match value {
            MyValue::Null => Value::Null,
            MyValue::Bool(v) => Value::Bool(v),
            MyValue::Number(number) => Value::Number(match number {
                Number::PosInt(n) => serde_json::Number::from(n),
                Number::NegInt(n) => serde_json::Number::from(n),
                Number::Float(n) => serde_json::Number::from_f64(n).unwrap(),
            }),
            MyValue::String(s) => Value::String(s),
            MyValue::Array(values) => Value::Array(values.into_iter().map(Value::from).collect()),
            MyValue::Object(btree_map) => Value::Object(
                btree_map
                    .into_iter()
                    .map(|v| (v.0, Value::from(v.1)))
                    .collect(),
            ),
        }
    }
}

#[cfg(feature = "json")]
impl From<Value> for MyValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => MyValue::Null,
            Value::Bool(b) => MyValue::Bool(b),
            Value::Number(number) => {
                if let Some(v) = number.as_f64() {
                    MyValue::Number(Number::Float(v))
                } else if let Some(v) = number.as_u64() {
                    MyValue::Number(Number::PosInt(v))
                } else if let Some(v) = number.as_i64() {
                    MyValue::Number(Number::NegInt(v))
                } else {
                    unreachable!()
                }
            }
            Value::String(s) => MyValue::String(s),
            Value::Array(values) => MyValue::Array(values.into_iter().map(MyValue::from).collect()),
            Value::Object(map) => {
                MyValue::Object(map.into_iter().map(|v| (v.0, MyValue::from(v.1))).collect())
            }
        }
    }
}

#[async_trait]
pub trait ResponseT: Send + Sync {
    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, ScraperError>;

    fn headers(&self) -> Vec<(&str, &str)>;

    fn status(&self) -> u16;
}
