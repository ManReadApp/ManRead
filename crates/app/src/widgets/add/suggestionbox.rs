use egui::util::hash;
use egui::{
    AboveOrBelow, Key, PopupCloseBehavior, Response, ScrollArea, TextBuffer, TextEdit, Ui, Widget,
};

pub struct SuggestionBox {
    id: String,
    pub items: Vec<String>,
    filter: Option<Box<dyn Fn(String, Vec<String>) -> Vec<String>>>,
    max_height: Option<f32>,
    closed: Option<u64>,
    open: Option<u64>,
    width: Option<f32>,
    popup_width: Option<f32>,
}

#[derive(PartialEq)]
enum KeyPressed {
    Esc,
    Enter,
}

impl SuggestionBox {
    pub fn new(id: impl ToString, data: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            items: data,
            filter: None,
            max_height: None,
            closed: None,
            open: None,
            width: None,
            popup_width: None,
        }
    }

    pub fn update_tags(&mut self, tags: Vec<String>) {
        self.items = tags
    }

    pub fn default_filter(mut self) -> Self {
        self.filter = Some(Box::new(|item, items| {
            items
                .into_iter()
                .filter(|v| v.to_lowercase().contains(&item.to_lowercase()))
                .collect()
        }));
        self
    }

    fn keybinds(&mut self, ui: &mut Ui, data: &str) -> Option<KeyPressed> {
        let mut res = None;
        ui.input(|v| {
            if v.key_pressed(Key::Escape) {
                self.closed = Some(hash(data));
                self.open = None;
                res = Some(KeyPressed::Esc);
            } else if v.key_pressed(Key::F1) {
                self.open = Some(hash(data));
                self.closed = None;
            } else if v.key_pressed(Key::Enter) {
                res = Some(KeyPressed::Enter);
            }
        });
        res
    }

    pub fn show_box<S: TextBuffer>(&mut self, ui: &mut Ui, text: &mut S) -> (Response, bool) {
        let id = ui.make_persistent_id(&self.id);
        let text_response = TextEdit::singleline(text)
            .desired_width(self.width.unwrap_or(ui.available_width()))
            .ui(ui);
        let mut closed = false;
        let t = text.as_str();

        let lost_focus = text_response.lost_focus();
        let enter = if text_response.has_focus() || lost_focus {
            let key = self.keybinds(ui, t);
            if key.is_some() && lost_focus {
                text_response.request_focus();
            }
            key == Some(KeyPressed::Enter)
        } else {
            false
        };

        let filtered = match &self.filter {
            None => self.items.clone(),
            Some(filter) => filter(text.as_str().to_string(), self.items.clone()),
        };
        if !enter {
            if filtered.is_empty() {
                closed = true;
            } else {
                if t.is_empty() {
                    closed = true;
                } else if let Some(v) = self.closed {
                    if hash(t) == v {
                        closed = true;
                    } else {
                        self.closed = None;
                    }
                }
                if closed {
                    if let Some(v) = self.open {
                        if hash(t) == v {
                            closed = false;
                        } else {
                            self.open = None;
                        }
                    }
                }
            }
        }

        ui.memory_mut(|mem| {
            if mem.is_popup_open(id) == closed {
                mem.toggle_popup(id);
            }
        });
        if closed {
            return (text_response, false);
        } else if enter {
            return (text_response, true);
        }
        let max_height = self.max_height.unwrap_or(ui.spacing().combo_height);

        let above_or_below =
            if ui.next_widget_position().y + ui.spacing().interact_size.y + max_height
                < ui.ctx().screen_rect().bottom()
            {
                AboveOrBelow::Below
            } else {
                AboveOrBelow::Above
            };
        let mut selected = -1;

        egui::popup::popup_above_or_below_widget(
            ui,
            id,
            &text_response,
            above_or_below,
            PopupCloseBehavior::CloseOnClick,
            |ui| {
                if let Some(wi) = self.popup_width {
                    ui.set_max_width(wi);
                }
                ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, |ui| {
                        for (index, item) in filtered.iter().enumerate() {
                            ui.selectable_value(&mut selected, index as i32, item);
                        }
                    })
                    .inner
            },
        );
        if selected != -1 {
            text.replace_with(filtered.get(selected as usize).expect("Shouldn't fail"));
            self.closed = Some(hash(text.as_str()));
            (text_response, true)
        } else {
            (text_response, false)
        }
    }
    pub fn set_width(&mut self, width: f32) {
        self.width = Some(width);
    }

    pub fn set_popup_width(&mut self, popup_width: f32) {
        self.popup_width = Some(popup_width);
    }
}
