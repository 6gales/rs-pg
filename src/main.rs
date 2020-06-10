#[macro_use]
extern crate rs_pg_derive;
extern crate rs_pg_scheme;

use rs_pg::database::{PostgresClient, CreateTableOptions, ConnectOptions};
use rs_pg::{Entity, Serial, Scheme, WithId, DbError};
use serde::{Deserialize, Serialize};

#[derive(Entity, Serialize, Deserialize)]
#[table_name = "persons"]
struct Person {
	#[primary_key]
	id: Serial,

	#[unique]
	first_name: String,

	age: i16,

	#[skip]
	#[serde(default)]
	useless_info: String
}

#[derive(Entity)]
#[check("person_id < 20 AND work_id < 100")]
struct Work {
	#[primary_key]
	work_id: Serial,

	#[references("persons", id)]
	#[on_delete(Cascade)]
	#[on_update(SetNull)]
	person_id: i32,

	#[check("salary > 100")]
	salary: i64,

	description: Option<String>
}

use chrono::naive::{NaiveDate, NaiveTime};

pub fn main() -> Result<(), DbError> {

	base_usage_example()?;
	Ok(())
}

fn base_usage_example() -> Result<(), DbError> {

	let opts = ConnectOptions{
		user: "postgres".to_string(),
		password: "zeratul".to_string(),
		address: "localhost".to_string(),
		port: 5432,
		database: "postgres".to_string()
	};

	let mut client = PostgresClient::connect_with_opts(&opts)?;
	//или так
	//let mut client = PostgresClient::connect("postgresql://postgres:zeratul@localhost:5432/postgres")?;
	
	client.create_table::<Person>(CreateTableOptions{temp: false, if_not_exists: true})?;
	client.create_table::<Work>(CreateTableOptions{temp: false, if_not_exists: true})?;

	//client.insert(&Person{id: 0, first_name:"Finn".to_string(), age:21, useless_info: "".to_string()})?;

	let persons = vec!(Person{id: 0, first_name:"And".to_string(), age:16, useless_info: "".to_string()},
					   Person{id: 0, first_name:"Nund".to_string(), age:19, useless_info: "".to_string()});
	client.insert_many(&persons)?;

	let p = client.select_by_pk::<_, Person>(1)?;
	println!("Person with id = {} {}", p.id, p.first_name);
	let v = client.select_all::<Person>()?;
	println!("All persons:");
	for pe in v {
		println!("{} {}", pe.id, pe.first_name);
	}

	client.delete_by_pk::<_, Person>(1)?;
	let v = client.select_all::<Person>()?;
	println!("All persons after delete:");
	for pe in v {
		println!("{} {}", pe.id, pe.first_name);
	}
	Ok(())
}