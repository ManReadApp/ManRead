use egui::{pos2, vec2, AboveOrBelow, Color32, ScrollArea, Sense, Ui};

pub struct ThreeDot {
    id: String,
    size: f32,
    color: Color32,
}

impl ThreeDot {
    pub fn new<T: ToString>(id: T, size: f32, dark: bool) -> ThreeDot {
        Self {
            id: id.to_string(),
            size,
            color: match dark {
                true => Color32::WHITE,
                false => Color32::BLACK,
            },
        }
    }
}

impl ThreeDot {
    pub fn show<R, V>(
        self,
        ui: &mut Ui,
        mut data: V,
        add_contents: impl FnOnce(&mut Ui, &mut V) -> R,
    ) -> V {
        let id = ui.make_persistent_id(self.id);
        let size = vec2(self.size, self.size);
        let r = self.size * 0.1;
        let (rect, re) = ui.allocate_at_least(size, Sense::click());
        let pos = rect.center();
        let top_pos = pos2(pos.x, rect.center_top().y + r);
        let bottom_pos = pos2(pos.x, rect.center_bottom().y - r);
        ui.painter().circle_filled(top_pos, r, self.color);
        ui.painter().circle_filled(pos, r, self.color);
        ui.painter().circle_filled(bottom_pos, r, self.color);
        let max_height = 200.0;

        let above_or_below =
            if ui.next_widget_position().y + ui.spacing().interact_size.y + max_height
                < ui.ctx().screen_rect().bottom()
            {
                AboveOrBelow::Below
            } else {
                AboveOrBelow::Above
            };

        if re.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(id));
        }

        egui::popup::popup_above_or_below_widget(ui, id, &re, above_or_below, |ui| {
            ui.set_max_width(100.0);
            ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| add_contents(ui, &mut data))
                .inner
        });
        data
    }
}
