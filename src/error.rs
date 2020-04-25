#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Database url parse error: {0}")]
    DbUrlParseError(#[from] url::ParseError),
}

#[derive(Debug, Error)]
#[error("{0}")]
pub enum PlatformError {
    #[cfg(feature = "with-postgres")]
    #[error("{0}")]
    PostgresError(#[from] PostgresError),
    #[cfg(feature = "with-sqlite")]
    #[error("{0}")]
    SqliteError(#[from] SqliteError),
    #[cfg(feature = "with-mysql")]
    #[error("{0}")]
    MysqlError(#[from] MysqlError),
}

#[cfg(feature = "with-postgres")]
impl From<PostgresError> for DbError {
    fn from(e: PostgresError) -> Self { DbError::PlatformError(PlatformError::from(e)) }
}

#[cfg(feature = "with-sqlite")]
impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError::PlatformError(PlatformError::SqliteError(SqliteError::from(e)))
    }
}

#[cfg(feature = "with-sqlite")]
impl From<SqliteError> for DbError {
    fn from(e: SqliteError) -> Self { DbError::PlatformError(PlatformError::from(e)) }
}

#[cfg(feature = "with-mysql")]
impl From<MysqlError> for DbError {
    fn from(e: MysqlError) -> Self { DbError::PlatformError(PlatformError::from(e)) }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Sql injection attempt error: {0}")]
    SqlInjectionAttempt(String),
    #[error("{0}")]
    DataError(DataError),
    #[error("{0}")]
    PlatformError(PlatformError),
    #[error("{0}")]
    ConvertError(ConvertError),
    #[error("{0}")]
    ConnectError(ConnectError), //agnostic connection error
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