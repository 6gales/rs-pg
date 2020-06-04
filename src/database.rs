extern crate postgres;

use postgres::{Client, NoTls, ToStatement};//, Error};
use postgres::types::ToSql;

use serde::{
	//Deserialize, 
	Serialize};

use std::result::Result;
use crate::entity::Entity;
use rs_pg_scheme::pg_type_to_str;
//use crate::error::DbError;

pub struct CreateTableOptions {
	pub temp: bool,
	pub if_not_exists: bool
}

pub struct PostgresClient {
	client: Client,
}

impl PostgresClient {
	pub fn connect(conn_string: &str) -> Result<PostgresClient, postgres::Error> {
		let client = Client::connect(conn_string, NoTls)?;
		Ok(PostgresClient{
			client: client,
		})
	}

	pub fn batch_execute(&mut self, query: &str) -> Result<(), postgres::Error> {
		self.client.batch_execute(query)
	}

	pub fn execute<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64, postgres::Error> where T: ToStatement {
		self.client.execute(query, params)
	}

	pub fn insert<T: Serialize>(_item: T) {
		
	}

	pub fn create_table<T: Entity>(&mut self, opts: CreateTableOptions) -> Result<(), postgres::Error> {

		let mut query = String::from("CREATE ");
		if opts.temp {
			query += "TEMP ";
		}
		query += "TABLE ";
		if opts.if_not_exists {
			query += "IF NOT EXISTS ";
		}
		let scheme = T::scheme();
		query += scheme.name.as_str();
		query += "(";
		
		for name in scheme.fields.keys() {
			query += name.as_str();
			query += " ";

			let field = scheme.fields.get(name).unwrap();
			query += pg_type_to_str(&field.ty);
			query += " ";

			for constr in field.constraints.iter() {
				query += constr.to_string().as_str();
				query += " ";
			}
			query += ","
		}
		query.pop();
		query += ")";

		println!("Query: \"{}\"", query);
		self.client.batch_execute(query.as_str())
	}

	pub fn model() {

	}

	// pub fn create_table<'a, T: Entity<'a>>(&mut self, ex: T) -> Result<(), DbError> {

	// 	let val = serde_json::to_value(ex).unwrap();
	// 	let mut query = String::from("CREATE TABLE ") + ex.name() + "(";

	// 	if let Value::Object(fields) = val {
			
	// 		for (field, _) in fields {
	// 			query += field.as_str() + ;
	// 		}
	// 		self.client.batch_execute(query.as_str());
	// 		Ok(())

	// 	} else {
	// 		Error("Bad table type")
	// 	}
	// }
}