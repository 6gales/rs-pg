#[macro_use]
extern crate rs_pg_derive;
extern crate rs_pg_scheme;

use rs_pg::database::{PostgresClient, CreateTableOptions, ConnectOptions};
use rs_pg::{Entity, Serial, Scheme, WithId, DbError};
use serde::{Deserialize, Serialize};
use std::{
	net::IpAddr,
	net::Ipv4Addr,
	time::SystemTime
};
use time::{Time, Date};

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

#[derive(Entity, Serialize, Deserialize)]
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

#[derive(Entity, Serialize, Deserialize)]
struct NetworkInfo {
	#[primary_key]
	id: Serial,
	net_addr: IpAddr,
	last_updated: SystemTime,
}

use postgres::{Client, NoTls};


pub fn main() -> Result<(), DbError> {

	insert_with_return_usage_example()?;
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

	let mut ret = Person{id: 0, first_name:"Fiona".to_string(), age:21, useless_info: "".to_string()};
	client.insert_with_return(&mut ret)?;
	println!("Inserted and returned: {} {}", ret.id, ret.first_name);

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

fn insert_with_return_usage_example() -> Result<(), DbError> {

	let opts = ConnectOptions{
		user: "postgres".to_string(),
		password: "zeratul".to_string(),
		address: "localhost".to_string(),
		port: 5432,
		database: "postgres".to_string()
	};

	let mut client = PostgresClient::connect_with_opts(&opts)?;
	
	client.create_table::<Person>(CreateTableOptions{temp: false, if_not_exists: true})?;
	client.create_table::<Work>(CreateTableOptions{temp: false, if_not_exists: true})?;

	let mut ret = Person{id: 0, first_name:"Abcd".to_string(), age:21, useless_info: "".to_string()};
	client.insert_with_return(&mut ret)?;
	assert_ne!(0, ret.id);
	println!("Inserted and returned: {} {}", ret.id, ret.first_name);

	//создаём работу
	//None - чтобы вставить NULL, Some("very interesting job".to_string()) - чтобы вставить строку
	let mut ret_w = Work{work_id: 0, person_id: ret.id, salary: 134, description: None};
	client.insert_with_return(&mut ret_w)?;
	assert_ne!(0, ret_w.work_id);
	println!("Add work with id = {} for person with id = {} {}", ret_w.work_id, ret.id, ret.first_name);

	//попробуем создать работу, ссылающуюся на человека которого не существует
	let mut w = Work{work_id: 0, person_id: 0, salary: 12345, description: Some("unable to insert".to_string())};
	let res = client.insert_with_return(&mut w);
	match res {
		Ok(_) => panic!("Should be error!"),
		Err(e) => println!("Found error {}, as expected", e),
	}

	//попробуем создать работу, нарушающую ограничение триггера
	let mut w = Work{work_id: 0, person_id: 0, salary: 50, description: Some("lala".to_string())};
	let res = client.insert_with_return(&mut w);
	match res {
		Ok(_) => panic!("Should be error!"),
		Err(e) => println!("Found error \"{}\", as expected", e),
	}

	let mut persons = vec!(Person{id: 0, first_name:"LALALa".to_string(), age:16, useless_info: "".to_string()},
					   Person{id: 0, first_name:"OKOKOK".to_string(), age:19, useless_info: "".to_string()});
	client.insert_many_with_return(&mut persons)?;
	for pe in persons {
		assert_ne!(0, pe.id);
		println!("{1} got id: {0}", pe.id, pe.first_name);
	}

	let p = client.select_by_pk::<_, Person>(ret.id)?;
	assert_eq!(ret.id, p.id);
	assert_eq!(ret.first_name, p.first_name);

	println!("Person with id = {} {}", p.id, p.first_name);
	let v = client.select_all::<Person>()?;
	println!("All persons:");
	for pe in v {
		println!("{} {}", pe.id, pe.first_name);
	}

	client.delete_by_pk::<_, Person>(ret.id)?;
	let v = client.select_all::<Person>()?;
	println!("All persons after delete:");
	for pe in v {
		println!("{} {}", pe.id, pe.first_name);
	}

	//попробуем достать работу, которую мы ранее добавили, так как мы удалили человека, она тоже должна удалиться
	let res = client.select_by_pk::<_, Work>(ret_w.work_id);
	match res {
		Ok(_) => panic!("Should be error!"),
		Err(e) => println!("Found error \"{}\", as expected", e),
	}
	Ok(())
}

fn time_inet_example() -> Result<(), DbError> {

	let opts = ConnectOptions{
		user: "postgres".to_string(),
		password: "zeratul".to_string(),
		address: "localhost".to_string(),
		port: 5432,
		database: "postgres".to_string()
	};

	let mut client = PostgresClient::connect_with_opts(&opts)?;
	
	client.create_table::<NetworkInfo>(CreateTableOptions{temp: false, if_not_exists: true})?;

	let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
	let t = SystemTime::now();
	let n = NetworkInfo{id: 0, net_addr: localhost_v4, last_updated: t};
	client.insert(&n)?;

	let persons = vec!(Person{id: 0, first_name:"And".to_string(), age:16, useless_info: "".to_string()},
					   Person{id: 0, first_name:"Nund".to_string(), age:19, useless_info: "".to_string()});
	client.insert_many(&persons)?;

	let selected = client.select_by_pk::<_, NetworkInfo>(1)?;
	println!("Network with id = {} {} {:?}", selected.id, selected.net_addr, selected.last_updated);
	let v = client.select_all::<NetworkInfo>()?;
	println!("All network infos:");
	for ni in v {
		println!("Network with id = {} {} {:?}", ni.id, ni.net_addr, ni.last_updated);
	}

	client.delete_by_pk::<_, NetworkInfo>(1)?;
	let v = client.select_all::<NetworkInfo>()?;
	println!("All network infos after delete:");
	for ni in v {
		println!("Network with id = {} {} {:?}", ni.id, ni.net_addr, ni.last_updated);
	}
	Ok(())
}