// #![feature(const_type_id)]

// extern crate type_info;

// #[macro_use]
// extern crate type_info_derive;

// use type_info::TypeInfo;

// #[derive(TypeInfo)]

use rs_pg::PostgresClient;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

pub fn main() {
//    create_table(Person);
}