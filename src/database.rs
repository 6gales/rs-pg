extern crate postgres;

use postgres::{Client, NoTls, Error};
use serde::{Deserialize, Serialize};
use serde_json::{
	Result,
	Value
};


// pub trait Database {
//     fn execute_sql_with_return(&mut self, sql: &str, param: &[&Value]) -> Result<Rows, DbError>;

//     fn get_table(&mut self, table_name: &TableName) -> Result<Table, DbError>;

//     fn get_all_tables(&mut self) -> Result<Vec<Table>, DbError>;

//     fn get_grouped_tables(&mut self) -> Result<Vec<SchemaContent>, DbError>;
// }

pub struct PostgresClient {
	client: Client,
}

impl PostgresClient {
	fn connect(conn_string: &str) -> Result<Self, Error> {
		let mut client = Client::connect(conn_string, NoTls)?;
		PostgresClient{
			client: client,
		}
	}

	fn batch_execute(&mut self, query: &str) -> Result<(), Error> {
		self.client.batch_execute(query)
	}

	pub fn create_table<T>(&mut self) where T : Serialize + Deserialize -> Result<(), Error> {

		let table = "test";//T::TABLE_NAME;
		let val = serde_json::to_value(T{})?
		let mut query = String::from("CREATE TABLE(");

		if let Value::Object(fields) = val {
			
			for (field, _) in fields {
//				query
			}
			self.client.batch_execute(query);
			Ok(())

		} else {
			Error("Bad table type")
		}
	}
}