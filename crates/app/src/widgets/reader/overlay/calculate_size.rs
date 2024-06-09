use crate::get_app_data;
use crate::widgets::reader::overlay::ReaderTranslationArea;
use eframe::emath::Rect;
use eframe::epaint::text::{LayoutJob, TextFormat};
use eframe::epaint::{FontFamily, FontId};
use egui::{Ui, WidgetText};

impl ReaderTranslationArea {
    pub(crate) fn get_size(&self, rect: Rect, ui: &mut Ui) -> WidgetText {
        let v = FontId::new(
            12.,
            FontFamily::Name(
                get_app_data()
                    .fonts
                    .lock()
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                    .into(),
            ),
        );
        let text = match self.translated_text.get("eng") {
            Some(v) => v,
            None => self.translated_text.values().next().unwrap(),
        }
        .to_string();

        let mut lj = LayoutJob::simple(text, v.clone(), self.text_color, rect.width());
        let mut fontsize = 12;
        //TODO: cache fontsize
        //TODO: add guess based on rect size
        //TODO: add check for extreme height and small width
        //TODO: find a good algorithm to calculate fontsize
        let font_range = [5, 20];
        let mut increase = true;
        let update_fontsize = |size: u32, lj: &mut LayoutJob| {
            lj.sections[0].format = TextFormat::simple(
                FontId::new(size as f32, v.family.clone()),
                lj.sections[0].format.color,
            )
        };
        let y = rect.size().y;
        let mut last_action = LastAction::None;
        loop {
            if fontsize >= font_range[1] || fontsize <= font_range[0] {
                break;
            }
            if increase {
                fontsize += 1;
            } else {
                fontsize -= 1;
            }
            update_fontsize(fontsize, &mut lj);
            let size = ui.fonts(|fonts| fonts.layout_job(lj.clone())).size();
            increase = y >= size.y;
            if last_action == LastAction::Increase && !increase {
                update_fontsize(fontsize - 1, &mut lj);
                break;
            } else if last_action == LastAction::Decrease && increase {
                update_fontsize(fontsize + 1, &mut lj);
                break;
            } else {
                last_action = LastAction::new(increase)
            }
        }
        WidgetText::from(lj)
    }
}

#[derive(PartialEq, Eq)]
enum LastAction {
    Decrease,
    Increase,
    None,
}

impl LastAction {
    fn new(increase: bool) -> Self {
        match increase {
            true => Self::Increase,
            false => Self::Decrease,
        }
    }
}
