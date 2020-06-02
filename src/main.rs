// #![feature(const_type_id)]

// extern crate type_info;

// #[macro_use]
// extern crate type_info_derive;

// use type_info::TypeInfo;

// #[derive(TypeInfo)]

//use rs_pg::database::PostgresClient;
//use rs_pg;
use serde::{Deserialize, Serialize};
//use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[macro_use]
extern crate rs_pg_derive;

#[derive(Serialize)]
struct Worker {
	id: i32,
	name: String
}

use rs_pg::database::{PostgresClient, CreateTableOptions};
use rs_pg::Entity;

#[derive(Entity)]
#[table_name = "toasts"]
struct FrenchToast {
	#[primary_key]
	id: i32,
	#[unique]
	name: String,
	x: f64,
	y: f64
}

#[derive(Entity)]
struct Waffles {
	#[primary_key]
	waffle_id: i32,
//	#[references(FrenchToast)]
	#[references("toasts", "id")]
	toast_id: i32,
	#[skip]
	taste: String
}

use postgres::{Client, NoTls};

pub fn main12() -> Result<(), postgres::Error> {

	// let mut client = PostgresClient::connect("postgresql://postgres:zeratul@localhost:5432/postgres")?;
	// client.create_table::<FrenchToast>(CreateTableOptions{temp: false, if_not_exists: true})?;
	// client.create_table::<Waffles>(CreateTableOptions{temp: false, if_not_exists: false})?;

	let mut client = Client::connect("postgresql://postgres:zeratul@localhost:5432/postgres", NoTls)?;

	// let bar = 1i32;
	// let baz = true;
	// let rows_updated = client.execute(
	// 	"UPDATE foo SET bar = $1 WHERE baz = $2",
	// 	&[&bar, &baz],
	// )?;

	// println!("{} rows updated", rows_updated);

	let worker = Person{age: 15, name: String::from("aaa")};
	let val = serde_json::to_value(worker).unwrap();

	println!("{}", val);
	let mut query = String::from("INSERT INTO person(");
	let map = val.as_object().unwrap();

	for pair in map {
		query += pair.0.as_str();
		query += ", ";
	}
	query.pop();
	query.pop();
	query += ") VALUES (";

	let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();
	let mut i = 1;
	let mut array: [i32; 3] = [0, 0, 0];
	let mut j = 0;
	for pair in map {
		query += "$";
		query += i.to_string().as_str();
		query += ", ";
		i += 1;

		match pair.1 {
			serde_json::Value::Bool(b) => {},//values.push(b),
		    serde_json::Value::Number(num) => {
				array[j] = num.as_i64().unwrap() as i32;
				j += 1;
			},
			serde_json::Value::String(s) => {}//values.push(s),
			_ => panic!("Unexpected type"),
		}
	}
	query.pop();
	query.pop();
	query += ")";

	j = 0;
	for pair in map {
		match pair.1 {
			serde_json::Value::Bool(b) => values.push(b),
		    serde_json::Value::Number(num) => {
				values.push(&array[j]);
				j += 1;
			},
			serde_json::Value::String(s) => values.push(s),
			_ => panic!("Unexpected type"),
		}
	}

	println!("{}", query);
	println!("{:?}", values);
	//client.execute("query: &T", params: &[&(dyn ToSql + Sync)])

    client.execute(query.as_str(), values.as_slice())?;
//    &[&"Jane", &23])?;

//     client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)",
// 	&[&"Alice", &32])?;
	
// 	let name = "Ferris";
// let data = None::<&[u8]>;
// client.execute(
//     "INSERT INTO person (name, data) VALUES ($1, $2)",
//     &[&name, &data],
// )?;
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
	Ok(())
}


// #[proc_macro_derive(FiniteStateMachine, attributes(state_transitions, state_change))]
// pub fn fxsm(input: TokenStream) -> TokenStream {
//     // ...
// }

// #[derive(Copy, Clone, Debug, FiniteStateMachine)]
// #[state_change(GameEvent, change_condition)] // optional
// enum GameState {
//     #[state_transitions(NeedServer, Ready)]
//     Prepare { players: u8 },
//     #[state_transitions(Prepare, Ready)]
//     NeedServer,
//     #[state_transitions(Prepare)]
//     Ready,
// }

// #[derive(Clone, Debug, Deserialize)]
// struct Person {
//     name: String,
//     age: i32,
// }
use rs_pg::from_row;

#[derive(Clone, Debug, Deserialize)]
struct NuRep {
    aaa: String,
	bbb: i32,
	ccc: Option<i32>,
}

// enum Constraint<T> {
// 	Unique(T),
// 	PrimaryKey(T),
// 	References(T)
// }

fn main() -> Result<(), Box<postgres::Error>> {
	let mut client = Client::connect("postgresql://postgres:zeratul@localhost:5432/postgres", NoTls)?;

    client.execute("CREATE TABLE IF NOT EXISTS NuRep (
        aaa VARCHAR NOT NULL,
		bbb INT NOT NULL,
		ccc INT NULL
    )", &[])?;

    // client.execute("INSERT INTO NuRep (aaa, bbb, ccc) VALUES ($1, $2, $3)",
    // &[&"Jane", &23, &117])?;

    // client.execute("INSERT INTO NuRep (aaa, bbb, ccc) VALUES ($1, $2, NULL)",
    // &[&"Alice", &32])?;
    
	let rows = client.query("SELECT aaa, bbb, ccc FROM NuRep", &[])?;
	
//	let people: Vec<Person> = serde_postgres::from_rows(&rows)?;
	let mut people: Vec<NuRep> = vec!();
	for row in rows {
		let person = from_row(row).unwrap();
		people.push(person);
		// let a: String = row.get(0);
		// let b: i32 = row.get(1);
		// let c: Option<i32> = row.get(2);
		// println!("{:?} {:?} {:?}", a, b, c);
	}

    for person in people {
        println!("{:?}", person);
    }

    Ok(())
}