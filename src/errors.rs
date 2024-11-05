use std::fmt::{Display, Formatter};

pub type PrResult<T> = Result<T, Error>;

pub enum Error {
    InsertingSummary(String, i32),
    UpdatingSummary(String, i32),
    FetchingCycles(String),
    InsertingBook(String, i32),
    UpdatingBook(String, i32),
    UpdatingUser(String, String),
    IncorrectPassword(String),
    UnknownUser(String),
    Unknown,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        let string = match self {
            InsertingSummary(s, n) => { format!("Error inserting summary {n}: {s}") }
            UpdatingSummary(s, n) => { format!("Error updating summary {n}: {s}") }
            FetchingCycles(s) => { format!("Error fetching cycles: {s}") }
            InsertingBook(s, n) => { format!("Error inserting book {n}: {s}") }
            UpdatingBook(s, n) => { format!("Error updating book {n}: {s}") }
            UpdatingUser(s, username) => { format!("Error user {username}: {s}") }
            IncorrectPassword(username) => { format!("Incorrect password for {username}") }
            UnknownUser(username) => { format!("Unknown user {username}") }
            Unknown => { format!("Unknown error") }
        };

        f.write_str(&string)
    }
}
