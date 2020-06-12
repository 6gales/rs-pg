//! When serializing or deserializing from Postgres rows goes wrong.
use std::fmt;
use serde::{de, ser};
use thiserror::Error;

impl From<postgres::Error> for DbError {
    fn from(e: postgres::Error) -> Self { DbError::PostgresError(e) }
}

impl From<DeError> for DbError {
	fn from(e: DeError) -> Self { DbError::ConvertError(e) }
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
    ConvertError(DeError),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// This type represents all possible error that can occur when deserializing
/// postgres rows.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum DeError {
    /// A custom defined error occured. Typically coming from `serde`.
	#[error("{0}")]
    Message(String),
    /// Row contained a field unknown to the data structure.
	#[error("Unknown field")]
	UnknownField,
    /// Row's column type was different from the Rust data structure.
	#[error("Invalid type")]
	InvalidType(String),
    /// Rust data structure contained a type unsupported by `serde_postgres`.
	#[error("Type unsupported")]
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

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Zero record returned")]
    ZeroRecordReturned,
    #[error("More than one record returned")]
	MoreThan1RecordReturned,
	#[error("Item vector is empty")]
	EmptyVector,
	#[error("Expected {0} records, {1} returned")]
	WrongNumberOfRecordsReturned(usize, usize)
}