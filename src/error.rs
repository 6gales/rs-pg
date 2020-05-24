//! When serializing or deserializing from Postgres rows goes wrong.
use std::{fmt, error};
use serde::{de, ser};
use thiserror::Error;

/// Alias for a `Result` with the error type `serde_postgres::Error`.
pub type Result<T> = ::std::result::Result<T, DeError>;

/// This type represents all possible error that can occur when deserializing
/// postgres rows.
#[derive(Clone, Debug, PartialEq)]
pub enum DeError {
    /// A custom defined error occured. Typically coming from `serde`.
    Message(String),
    /// Row contained a field unknown to the data structure.
    UnknownField,
    /// Row's column type was different from the Rust data structure.
    InvalidType(String),
    /// Rust data structure contained a type unsupported by `serde_postgres`.
    UnsupportedType,
}

impl de::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeError::Message(msg.to_string())
    }
}

impl ser::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeError::Message(msg.to_string())
    }
}

impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(error::Error::description(self))
    }
}

impl error::Error for DeError {
    fn description(&self) -> &str {
        match self {
            DeError::Message(ref msg) => msg,
            DeError::UnknownField => "Unknown field",
            DeError::InvalidType(_) => "Invalid type",
            DeError::UnsupportedType => "Type unsupported",
        }
    }
}

// #[derive(Debug, Error)]
// pub enum ParseError {
//     #[error("Database url parse error: {0}")]
//     DbUrlParseError(#[from] url::ParseError),
// }

impl From<postgres::Error> for DbError {
    fn from(e: postgres::Error) -> Self { DbError::PostgresError(e) }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Sql injection attempt error: {0}")]
    SqlInjectionAttempt(String),
    #[error("{0}")]
    DataError(DataError),
    #[error("{0}")]
    PostgresError(postgres::Error),
    #[error("{0}")]
    ConvertError(ConvertError),
    // #[error("{0}")]
    // ConnectError(ConnectError), //agnostic connection error
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("Unknown data type")]
    UnknownDataType,
    #[error("Unsupported data type {0}")]
    UnsupportedDataType(String),
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Zero record returned")]
    ZeroRecordReturned,
    #[error("More than one record returned")]
    MoreThan1RecordReturned,
}