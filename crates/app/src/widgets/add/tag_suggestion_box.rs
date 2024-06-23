use api_structure::models::manga::tag::{Tag, TagSex};
use api_structure::req::manga::tag::TagsRequest;
use egui::epaint::ahash::HashMap;
use egui::util::hash;
use egui::{ComboBox, TextBuffer, Ui};
use log::error;
use std::collections::HashSet;

use crate::get_app_data;
use crate::requests::{RequestImpl, TagsRequestFetcher};

use super::suggestionbox::SuggestionBox;

pub struct TagSuggestionBox {
    id: String,
    sb: SuggestionBox,
    tags: HashSet<Tag>,
    queries: HashMap<TagSex, HashSet<u64>>,
    pub query: String,
    requests: Vec<TagsRequestFetcher>,
    sex: TagSex,
}

impl TagSuggestionBox {
    pub fn new_tag(&mut self) -> Tag {
        Tag {
            tag: self.query.take(),
            sex: self.sex,
            description: None,
        }
    }
    pub fn new(id: impl ToString) -> Self {
        let new_id = id.to_string();
        let sb = SuggestionBox::new(id, vec![]).default_filter();
        Self {
            id: new_id,
            sb,
            tags: Default::default(),
            queries: TagSex::get_all()
                .into_iter()
                .map(|v| (v, HashSet::new()))
                .collect(),
            query: Default::default(),
            requests: vec![],
            sex: TagSex::Unknown,
        }
    }

    fn new_request(&mut self, ctx: egui::Context) {
        let mut r = TagsRequest::fetcher_ctx(&get_app_data().url, ctx);
        r.set_body(&TagsRequest {
            limit: 50,
            query: self.query.clone(),
            sex: self.sex,
        });
        r.send();
        self.requests.push(r);
    }

    fn filter_requests(&mut self) {
        let mut remove = vec![];
        for (index, item) in self.requests.iter_mut().enumerate() {
            if item.result().is_some() {
                remove.push(index)
            }
        }

        while let Some(v) = remove.pop() {
            let data = self.requests.remove(v);
            match data.take_result().unwrap() {
                crate::fetcher::Complete::Json(tags) => {
                    for item in tags {
                        self.tags.insert(item.clone());
                    }
                }
                _ => {
                    error!("unexpected result in fetching tags")
                }
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui) -> Option<Tag> {
        self.filter_requests();
        self.sb.update_tags(
            self.tags
                .iter()
                .filter_map(|v| match v.sex == self.sex {
                    true => Some(v.tag.clone()),
                    false => None,
                })
                .collect(),
        );
        let (response, take) = ui
            .horizontal(|ui| {
                ComboBox::from_id_source(format!("{}_combo", self.id))
                    .selected_text(self.sex.get_name())
                    .show_ui(ui, |ui| {
                        for v in TagSex::get_all() {
                            let text = if v == TagSex::None {
                                "○".to_string()
                            } else {
                                v.to_string()
                            };
                            ui.selectable_value(&mut self.sex, v, text)
                                .on_hover_text(v.get_name());
                        }
                    });
                self.sb.show_box(ui, &mut self.query)
            })
            .inner;

        if response.has_focus() || response.lost_focus() {
            let hash = hash(&self.query);
            if !self.queries.get(&self.sex).unwrap().contains(&hash) {
                self.queries.get_mut(&self.sex).unwrap().insert(hash);
                self.new_request(ui.ctx().clone());
            }
        }
        if take {
            Some(self.new_tag())
        } else {
            None
        }
    }
}
