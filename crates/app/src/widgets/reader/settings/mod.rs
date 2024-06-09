mod reader;
mod view;
pub(crate) struct Settings {
    pub(crate) reading_mode: ReadingMode,
    pub(crate) prefetch: f32,
    pub(crate) view_area: ViewArea,
    pub(crate) version_hierachy: Vec<String>,
}
pub use reader::ReadingMode;
pub use view::ViewArea;
pub use view::get_screen_dim;