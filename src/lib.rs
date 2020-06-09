pub mod database;
pub use database::{PostgresClient, CreateTableOptions, ConnectOptions};

pub mod entity;
pub use entity::{Entity, WithId};

extern crate serde;
extern crate postgres;
extern crate rs_pg_scheme;

pub mod de;
pub mod error;

pub use de::{from_row, Deserializer};
pub use error::{DbError};//, Result};
pub use rs_pg_scheme::{Serial, Scheme};