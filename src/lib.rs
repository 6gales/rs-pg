// #![feature(const_type_id)]

// extern crate type_info;

// #[macro_use]
// extern crate type_info_derive;

// use type_info::{
// 	TypeInfo,
// 	DynamicTypeInfo,
// 	Type,
// 	TypeId,
// 	Data,
// };

pub mod database;
pub use database::{PostgresClient, CreateTableOptions};

pub mod entity;
pub use entity::Entity;

extern crate serde;
extern crate postgres;

pub mod de;
pub mod error;

pub use de::{from_row, Deserializer};
pub use error::{DeError, Result};