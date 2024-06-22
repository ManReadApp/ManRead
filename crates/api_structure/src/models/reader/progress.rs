#[derive(Serialize, Deserialize, Clone)]
pub struct Progress {
    pub width_start: f64,
    pub width_end: f64,
    pub height_start: f64,
    pub height_end: f64,
}
