//! What every site needs whatever it is: scanning markup without a parser, and the fetch and errors that speak the SDK's wire.

pub mod text;

use sdk::guest::http_get;
use sdk::wire::{Error, ErrorKind};

pub fn get_text(url: &str) -> Result<String, Error> {
    String::from_utf8(http_get(url)?).map_err(|_| internal(format!("{url} is not text")))
}

pub fn internal(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Internal, message)
}

pub fn not_found(id: &str) -> Error {
    Error::new(ErrorKind::NotFound, format!("no such map or title: {id}"))
}
