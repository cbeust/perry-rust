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
    UnknownUser(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Error::InsertingSummary(s, n) => { format!("Error inserting summary {n}: {s}") }
            Error::UpdatingSummary(s, n) => { format!("Error updating summary {n}: {s}") }
            Error::FetchingCycles(s) => { format!("Error fetching cycles: {s}") }
            Error::InsertingBook(s, n) => { format!("Error inserting book {n}: {s}") }
            Error::UpdatingBook(s, n) => { format!("Error updating book {n}: {s}") }
            Error::UpdatingUser(s, username) => { format!("Error user {username}: {s}") }
            Error::IncorrectPassword(username) => { format!("Incorrect password for {username}") }
            Error::UnknownUser(username) => { format!("Unknown user {username}") }
        };

        f.write_str(&string)
    }
}
