use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::v1::Status;

pub trait DisplaySearch: DeserializeOwned + Send {
    fn image_number(&self) -> u32;
    fn internal(&self) -> bool;
    fn id_url(&self) -> &String;
    fn ext(&self) -> Cow<String>;
    fn status(&self) -> Cow<Status>;
    fn titles(&self) -> Cow<HashMap<String, Vec<String>>>;
    fn cover(&self) -> &str;
}
