use std::fmt::{Display, Formatter};
use crate::entities::Summary;
use crate::errors::OkContent::Redirect;

pub type DbResult<T> = Result<T, Error>;

pub enum OkContent {
    Root,
    Ok,
    Html(String),
    Json(String),
    Image(Vec<u8>),
    Redirect(String),
}

pub type PrResult = Result<OkContent, Error>;

pub struct PrResultBuilder;

impl PrResultBuilder {
    pub fn html(s: String) -> PrResult {
        Ok(OkContent::Html(s))
    }

    pub fn json(s: String) -> PrResult {
        Ok(OkContent::Json(s))
    }

    pub fn root() -> PrResult {
        Ok(OkContent::Root)
    }

    pub fn ok() -> PrResult {
        Ok(OkContent::Ok)
    }

    pub fn image(image: Vec<u8>) -> PrResult {
        Ok(OkContent::Image(image))
    }

    pub fn redirect(url: String) -> PrResult {
        Ok(Redirect(url))
    }
}

#[derive(Debug)]
pub enum Error {
    InsertingSummary(String, i32),
    UpdatingSummary(String, i32),
    FetchingCycles(String),
    InsertingBook(String, i32),
    UpdatingBook(String, i32),
    UpdatingUser(String, String),
    IncorrectPassword(String),
    UnknownUser(String),
    InsertingInPending(String, Summary),
    InsertingCoverImage(String, i32),
    EmailError(String),
    PerryPediaCouldNotFind(i32),
    CouldNotFindCoverImage(String, i32),
    UnknownCoverImageError(i32),
    DeletingCover(String, i32),
    Unknown(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        let string = match self {
            InsertingSummary(e, n) => { format!("Error inserting summary {n}: {e}") }
            UpdatingSummary(e, n) => { format!("Error updating summary {n}: {e}") }
            FetchingCycles(e) => { format!("Error fetching cycles: {e}") }
            InsertingBook(e, n) => { format!("Error inserting book {n}: {e}") }
            UpdatingBook(e, n) => { format!("Error updating book {n}: {e}") }
            UpdatingUser(e, username) => { format!("Error updating user {username}: {e}") }
            IncorrectPassword(username) => { format!("Incorrect password for {username}") }
            UnknownUser(username) => { format!("Unknown user {username}") }
            InsertingCoverImage(e, n) => { format!("Error inserting cover image for book {n}: {e}") }
            InsertingInPending(e, summary) => { format!("Couldn't insert #{} into PENDING: {e}",
                summary.number) }
            EmailError(e) => { format!("Couldn't send email: {e}") }
            PerryPediaCouldNotFind(n) => { format!("PerryPedia: could not find {n}") }
            CouldNotFindCoverImage(e, n) => { format!("Couldn't load cover image for {n}: {e}") }
            UnknownCoverImageError(n) => { format!("Couldn't load cover image for {n}") }
            DeletingCover(e, n) => { format!("Couldn't delete cover {n}: {e}") }
            Unknown(s) => { format!("Unknown error: {s}") }
        };

        f.write_str(&string)
    }
}
