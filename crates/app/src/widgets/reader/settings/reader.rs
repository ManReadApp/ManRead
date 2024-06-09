#[allow(dead_code)]
//TODO:
pub enum ReadingMode {
    Single,
    /// bool for left to right
    Double(bool),
    Strip,
    /// bool for left to right
    Row(bool),
}
