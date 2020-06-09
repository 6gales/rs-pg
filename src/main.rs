use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[macro_use]
extern crate rs_pg_derive;
extern crate rs_pg_scheme;


#[derive(Serialize)]
struct Worker {
	id: i32,
	name: String
}

use rs_pg::database::{PostgresClient, CreateTableOptions, ConnectOptions};
use rs_pg::{Entity, Serial, Scheme, WithId, DbError};

#[derive(Entity, Serialize, Deserialize)]
#[table_name = "toasts"]
struct FrenchToast {
	#[primary_key]
	id: Serial,
	#[unique]
	name: String,
	x: f64,
	y: f64
}

#[derive(Entity)]
#[check("toast_id < 20 AND waffle_id < 100")]
struct Waffles {
	#[primary_key]
	waffle_id: Serial,
	#[references(FrenchToast, id)]
//	#[references("toasts", "id")]
	#[on_delete(Cascade)]
	#[on_update(SetNull)]
	#[check("toast_id > 11")]
	toast_id: i32,

	taste: Option<String>
}

pub fn main() -> Result<(), DbError> {

	let opts = ConnectOptions{
		user: "postgres".to_string(),
		password: "zeratul".to_string(),
		address: "localhost".to_string(),
		port: 5432,
		database: "postgres".to_string()
	};

	let mut client = PostgresClient::connect_with_opts(&opts)?;
	//let mut client = PostgresClient::connect("postgresql://postgres:zeratul@localhost:5432/postgres")?;
	client.create_table::<FrenchToast>(CreateTableOptions{temp: false, if_not_exists: true})?;
	client.create_table::<Waffles>(CreateTableOptions{temp: false, if_not_exists: true})?;

	let mut t = FrenchToast{id: 0, name: "ja".to_string(), x: 0.11, y: 0.66};
	client.insert_with_return(&mut t)?;
	println!("Id = {}", t.id);
	Ok(())
}