use std::fmt::{Display, Formatter};

pub type PrResult<T> = Result<T, Error>;

pub enum Error {
    InsertingSummary(String, i32),
    UpdatingSummary(String, i32),
    FetchingCycles(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Error::InsertingSummary(s, n) => {
                format!("Error inserting summary {n}: {s}")
            }
            Error::UpdatingSummary(s, n) => {
                format!("Error inserting summary {n}: {s}")
            }
            Error::FetchingCycles(s) => {
                format!("Error fetching cycles: {s}")
            }
        };
        f.write_str(&string)
    }
}
