// // #![feature(const_type_id)]

// // extern crate type_info;

// // #[macro_use]
// // extern crate type_info_derive;

// // use type_info::TypeInfo;

// // #[derive(TypeInfo)]

// //use rs_pg::database::PostgresClient;
// //use rs_pg;
// //use serde::{Deserialize, Serialize};
// //use serde_json::Result;

// // #[derive(Serialize, Deserialize)]
// // struct Person {
// //     name: String,
// //     age: u32,
// // }

// struct Worker {
// 	id: i32,
// 	name: String
// }

// extern crate postgres;

use postgres::{Client, NoTls, Error};
// use postgres::{Connection, TlsMode};
// use std::result::Result;


// pub fn main() {
// //    create_table(Person);

// 	// let mut client = Client::connect("postgresql://postgres:zeratul@localhost:5432/postgres", NoTls)?;
// 	let conn = Connection::connect("postgresql://postgres:zeratul@localhost:5432/postgres", TlsMode::None)
//             .unwrap();

// 	let mut json = String::from("{\"");
	
// 	for row in &conn.query("SELECT worker_id, name FROM workers", &[]).unwrap() {
// 	// for row in client.query("SELECT worker_id, name FROM workers", &[])? {

// 		for i in 0..row.len() {
// 			json.push_str(row.columns()[i].name());
// 			json.push_str("\":");
// 			let val: serde_json::Value = serde_json::from_value(row.get(i)).expect("Unable to des");
// 			val.to_string();
// //			json.push_str()
// 			json.push_str("\"");
// //			val.to_string();
// 		}
// 		// worker.id.to_string();
//         // println!("Worker #{} is {}", worker.id, worker.name);
// 	}
// 	println!("{}", json);

// }

extern crate serde;
extern crate postgres;


use serde::{Deserialize, Serialize};
//use postgres::{Connection, TlsMode};
use rs_pg::from_row;

#[derive(Clone, Debug, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

fn main() -> Result<(), Box<Error>> {
	let mut client = Client::connect("postgresql://postgres:zeratul@localhost:5432/postgres", NoTls)?;

    client.execute("CREATE TABLE IF NOT EXISTS Person (
        name VARCHAR NOT NULL,
        age INT NOT NULL
    )", &[])?;

    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
    &[&"Jane", &23])?;

    client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
    &[&"Alice", &32])?;
    
    let rows = client.query("SELECT name, age FROM Person", &[])?;

//	let people: Vec<Person> = serde_postgres::from_rows(&rows)?;
	let mut people: Vec<Person> = vec!();
	for row in rows {
		let person = from_row(row).unwrap();
		people.push(person);
	}

    for person in people {
        println!("{:?}", person);
    }

    Ok(())
}