pub struct Urls;

const CYCLES: &str = &"cycles";

impl Urls {
    pub fn cycles(number: i32) -> String {
        format!("/{CYCLES}/{number}")
    }
}
