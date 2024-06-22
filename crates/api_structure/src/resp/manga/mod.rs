use crate::models::manga::tag::Tag;

pub mod home;
pub mod info;
pub mod search;

pub type KindsResponse = Vec<String>;
pub type TagsResponse = Vec<Tag>;
