use api_structure::models::manga::tag::Tag;
use egui::Ui;

use super::{group::Group, tag_suggestion_box::TagSuggestionBox};

pub fn tag_field(ui: &mut Ui, tags: &mut Vec<Tag>, tag_field: &mut TagSuggestionBox) {
    if let Some(v) = Group::new(tags).ui(ui) {
        let tag = tags.remove(v);
        let pushtag = tag_field.new_tag();
        if !pushtag.tag.is_empty() {
            tags.push(pushtag);
        }
        tag_field.query = tag.tag;
    }
    if let Some(v) = tag_field.render(ui) {
        if !v.tag.is_empty() {
            tags.push(v);
        }
    }
}
