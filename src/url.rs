pub struct Urls;

pub const CYCLES: &str = &"cycles";
pub const SUMMARIES: &str = &"summaries";

impl Urls {
    pub fn cycles(number: i32) -> String {
        format!("/{CYCLES}/{number}")
    }
    pub fn summary(number: i32) -> String { format!("/{SUMMARIES}/{number}") }
    pub fn root() -> String { "/".into() }
}
