extern crate postgres;

use postgres::{Client, NoTls, ToStatement};
use postgres::types::{ToSql, FromSql};

use serde::{
	Deserialize, 
	Serialize};

use std::{
	result::Result,
	net::IpAddr,
	time::SystemTime
};
use crate::entity::{Entity, WithId};
use crate::de::from_row;
use crate::error::{DbError, DataError};

use rs_pg_scheme::{PgType, pg_type_to_str};

pub struct CreateTableOptions {
	pub temp: bool,
	pub if_not_exists: bool
}

pub struct ConnectOptions {
	pub user: String,
	pub password: String,
	pub address: String,
	pub port: i32,
	pub database: String
}

pub struct PostgresClient {
	client: Client,
}

impl PostgresClient {
	pub fn connect_with_str(conn_string: &str) -> Result<PostgresClient, DbError> {
		let client = Client::connect(conn_string, NoTls)?;
		Ok(PostgresClient{
			client: client,
		})
	}

	pub fn connect_with_opts(conn_opts: &ConnectOptions) -> Result<PostgresClient, DbError> {
		let client = Client::connect(format!("postgresql://{}:{}@{}:{}/{}",
			conn_opts.user, conn_opts.password, conn_opts.address, conn_opts.port, conn_opts.database).as_str(), NoTls)?;

		Ok(PostgresClient{
			client: client,
		})
	}

	pub fn batch_execute(&mut self, query: &str) -> Result<(), DbError> {
		self.client.batch_execute(query)?;
		Ok(())
	}

	pub fn execute<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64, DbError> where T: ToStatement {
		let rows_affected = self.client.execute(query, params)?;
		Ok(rows_affected)
	}

	pub fn create_table<T: Entity>(&mut self, opts: CreateTableOptions) -> Result<(), DbError> {

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
		self.client.batch_execute(query.as_str())?;
		Ok(())
	}

	pub fn insert<T: Entity + Serialize>(&mut self, item: &T) -> Result<(), DbError> {
		
		let scheme = T::scheme();
		let val = serde_json::to_value(item).unwrap();
	
		println!("{}", val);
		let mut query = String::from("INSERT INTO ");
		query += scheme.name.as_str();
		query += "(";
		//TODO: error handling
		let map = val.as_object().unwrap();
	
		for pair in map {
			if scheme.fields.contains_key(pair.0) {
				query += pair.0.as_str();
				query += ", ";
			}
		}
		query.pop();
		query.pop();
		query += ") VALUES (";
	
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();
		let mut tss: Vec<SystemTime> = vec!();
		let mut addrs: Vec<IpAddr> = vec!();

		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				query += "NULL, "
			} else if field.ty == PgType::Serial {
				query += "DEFAULT, "
			} else {
				query += "$";
				query += value_num.to_string().as_str();
				query += ", ";
				value_num += 1;

				match field.ty {
					PgType::Real => f32s.push(unwrap_num(pair.1).as_f64().unwrap() as f32),
					PgType::DoublePrecision => f64s.push(unwrap_num(pair.1).as_f64().unwrap()),
					PgType::Char => i8s.push(unwrap_num(pair.1).as_i64().unwrap() as i8),
					PgType::SmallInt => i16s.push(unwrap_num(pair.1).as_i64().unwrap() as i16),
					PgType::Integer => i32s.push(unwrap_num(pair.1).as_i64().unwrap() as i32),
					PgType::BigInt => i64s.push(unwrap_num(pair.1).as_i64().unwrap()),
					PgType::IpAddr => addrs.push(serde_json::from_value(pair.1.clone()).unwrap()),
					PgType::TimeStamp => tss.push(serde_json::from_value(pair.1.clone()).unwrap()),
					_ => {},
				};
			}
		}
		query.pop();
		query.pop();
		query += ")";

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();

		let mut ts_iter = tss.iter();
		let mut addr_iter = addrs.iter();
	
		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				continue;
			}
			if field.ty == PgType::Serial {
				continue;
			}

			match field.ty {
				PgType::Real => values.push(f32_iter.next().unwrap()),
				PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
				PgType::Char => values.push(i8_iter.next().unwrap()),
				PgType::SmallInt => values.push(i16_iter.next().unwrap()),
				PgType::Integer => values.push(i32_iter.next().unwrap()),
				PgType::BigInt => values.push(i64_iter.next().unwrap()),
				PgType::TimeStamp => values.push(ts_iter.next().unwrap()),
				PgType::IpAddr => values.push(addr_iter.next().unwrap()),
				PgType::Boolean => {
					if let serde_json::Value::Bool(b) = pair.1 {
						values.push(b);
					} else {
						panic!("Expected bool, found {}", pair.1);
					}
				},
				PgType::Text => {
					if let serde_json::Value::String(s) = pair.1 {
						values.push(s);
					} else {
						panic!("Expected string, found {}", pair.1);
					}
				},
				_ => {},
			};
		}
	
		println!("{}", query);
		println!("{:?}", values);
	
		self.client.execute(query.as_str(), values.as_slice())?;
		Ok(())
	}

	pub fn insert_many<T: Entity + Serialize>(&mut self, items: &Vec<T>) -> Result<(), DbError> {

		if items.len() == 0 {
			return Result::Err(DbError::DataError(DataError::EmptyVector));
		}
		let scheme = T::scheme();

		let val = serde_json::to_value(&items[0]).unwrap();	
		println!("{}", val);
		let mut query = String::from("INSERT INTO ");
		query += scheme.name.as_str();
		query += "(";
		//TODO: error handling
		let map = val.as_object().unwrap();
	
		for pair in map {
			if scheme.fields.contains_key(pair.0) {
				query += pair.0.as_str();
				query += ", ";
			}
		}

		query.pop();
		query.pop();
		query += ") VALUES ";
	
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();
		
		let mut tss: Vec<SystemTime> = vec!();
		let mut addrs: Vec<IpAddr> = vec!();
		
		for item in items {
			query += "(";
			let val = serde_json::to_value(item).unwrap();	
			//TODO: error handling
			let map = val.as_object().unwrap();

			for pair in map {
				let opt_field = scheme.fields.get(pair.0);
				if let None = opt_field {
					continue;
				}
				let field = opt_field.unwrap();

				if let serde_json::Value::Null = pair.1 {
					query += "NULL, "
				} else if field.ty == PgType::Serial {
					query += "DEFAULT, "
				} else {
					query += "$";
					query += value_num.to_string().as_str();
					query += ", ";
					value_num += 1;
			
					match field.ty {
						PgType::Real => f32s.push(unwrap_num(pair.1).as_f64().unwrap() as f32),
						PgType::DoublePrecision => f64s.push(unwrap_num(pair.1).as_f64().unwrap()),
						PgType::Char => i8s.push(unwrap_num(pair.1).as_i64().unwrap() as i8),
						PgType::SmallInt => i16s.push(unwrap_num(pair.1).as_i64().unwrap() as i16),
						PgType::Integer => i32s.push(unwrap_num(pair.1).as_i64().unwrap() as i32),
						PgType::BigInt => i64s.push(unwrap_num(pair.1).as_i64().unwrap()),
						PgType::IpAddr => addrs.push(serde_json::from_value(pair.1.clone()).unwrap()),
						PgType::TimeStamp => tss.push(serde_json::from_value(pair.1.clone()).unwrap()),
						_ => {},
					};
				}
			}
			query.pop();
			query.pop();
			query += "),";
		}
		query.pop();

		let mut j_values = vec!();
		let mut j_maps = vec!();
		for item in items {
			let val = serde_json::to_value(item).unwrap();
			j_values.push(val);
		}
		for i in 0..j_values.len() {
			let map = j_values[i].as_object().unwrap();
			j_maps.push(map);
		}

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();

		let mut ts_iter = tss.iter();
		let mut addr_iter = addrs.iter();

		for map in j_maps {
			for pair in map {
				let opt_field = scheme.fields.get(pair.0);
				if let None = opt_field {
					continue;
				}
				let field = opt_field.unwrap();

				if let serde_json::Value::Null = pair.1 {
					continue;
				}
				if field.ty == PgType::Serial {
					continue;
				}

				match field.ty {
					PgType::Real => values.push(f32_iter.next().unwrap()),
					PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
					PgType::Char => values.push(i8_iter.next().unwrap()),
					PgType::SmallInt => values.push(i16_iter.next().unwrap()),
					PgType::Integer => values.push(i32_iter.next().unwrap()),
					PgType::BigInt => values.push(i64_iter.next().unwrap()),
					PgType::TimeStamp => values.push(ts_iter.next().unwrap()),
					PgType::IpAddr => values.push(addr_iter.next().unwrap()),
					PgType::Boolean => {
						if let serde_json::Value::Bool(b) = pair.1 {
							values.push(b);
						} else {
							panic!("Expected bool, found {}", pair.1);
						}
					},
					PgType::Text => {
						if let serde_json::Value::String(s) = pair.1 {
							values.push(s);
						} else {
							panic!("Expected string, found {}", pair.1);
						}
					},
					_ => {},
				};
			}
		}
	
		println!("{}", query);
		println!("{:?}", values);
	
		self.client.execute(query.as_str(), values.as_slice())?;
		Ok(())
	}

	pub fn insert_with_return<'b, P, T>(&mut self, item: &mut T) -> Result<(), DbError>
	where P: for<'a> FromSql<'a> + ToSql,
	      T: Entity + WithId<'b, P> + Serialize {

		let scheme = T::scheme();
		let val = serde_json::to_value(&item).unwrap();
	
		println!("{}", val);
		let mut query = String::from("INSERT INTO ");
		query += scheme.name.as_str();
		query += "(";
		//TODO: error handling
		let map = val.as_object().unwrap();
	
		for pair in map {
			if scheme.fields.contains_key(pair.0) {
				query += pair.0.as_str();
				query += ", ";
			}
		}
		query.pop();
		query.pop();
		query += ") VALUES (";
	
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();
		
		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				query += "NULL, "
			} else if field.ty == PgType::Serial {
				query += "DEFAULT, "
			} else {
				query += "$";
				query += value_num.to_string().as_str();
				query += ", ";
				value_num += 1;
		
				if let serde_json::Value::Number(num) = pair.1 {
					match field.ty {
						PgType::Real => f32s.push(num.as_f64().unwrap() as f32),
						PgType::DoublePrecision => f64s.push(num.as_f64().unwrap()),
						PgType::Char => i8s.push(num.as_i64().unwrap() as i8),
						PgType::SmallInt => i16s.push(num.as_i64().unwrap() as i16),
						PgType::Integer => i32s.push(num.as_i64().unwrap() as i32),
						PgType::BigInt => i64s.push(num.as_i64().unwrap()),
						_ => {},
					};
				}
			}
		}
		query.pop();
		query.pop();
		query += ") RETURNING ";
		query += scheme.pk_field.unwrap().name.as_str();

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();
	
		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				continue;
			}
			if field.ty == PgType::Serial {
				continue;
			}

			match pair.1 {
				serde_json::Value::Bool(b) => values.push(b),
				serde_json::Value::Number(_) => {
					match field.ty {
						PgType::Real => values.push(f32_iter.next().unwrap()),
						PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
						PgType::Char => values.push(i8_iter.next().unwrap()),
						PgType::SmallInt => values.push(i16_iter.next().unwrap()),
						PgType::Integer => values.push(i32_iter.next().unwrap()),
						PgType::BigInt => values.push(i64_iter.next().unwrap()),
						_ => {},
					};
				},
				serde_json::Value::String(s) => values.push(s),
				_ => panic!("Unexpected type"),
			}
		}
	
		println!("{}", query);
		println!("{:?}", values);
	
		let rows = self.client.query(query.as_str(), values.as_slice())?;

		if rows.len() == 0 {
			Result::Err(DbError::DataError(DataError::ZeroRecordReturned))
		} else if rows.len() > 1 {
			Result::Err(DbError::DataError(DataError::MoreThan1RecordReturned))
		} else {
			let id: P = rows[0].get(0);
			item.__set_pk(id);
			Ok(())
		}
	}

	pub fn insert_many_with_return<'b, P, T>(&mut self, items: &mut Vec<T>) -> Result<(), DbError> 
	where P: for<'a> FromSql<'a> + ToSql,
		  T: Entity + WithId<'b, P> + Serialize {

		if items.len() == 0 {
			return Result::Err(DbError::DataError(DataError::EmptyVector));
		}
		let scheme = T::scheme();

		let val = serde_json::to_value(&items[0]).unwrap();	
		println!("{}", val);
		let mut query = String::from("INSERT INTO ");
		query += scheme.name.as_str();
		query += "(";
		//TODO: error handling
		let map = val.as_object().unwrap();
	
		for pair in map {
			if scheme.fields.contains_key(pair.0) {
				query += pair.0.as_str();
				query += ", ";
			}
		}

		query.pop();
		query.pop();
		query += ") VALUES ";
	
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();
		
		let mut tss: Vec<SystemTime> = vec!();
		let mut addrs: Vec<IpAddr> = vec!();
		
		for i in 0..items.len() {
			query += "(";
			let val = serde_json::to_value(&items[i]).unwrap();	
			//TODO: error handling
			let map = val.as_object().unwrap();

			for pair in map {
				let opt_field = scheme.fields.get(pair.0);
				if let None = opt_field {
					continue;
				}
				let field = opt_field.unwrap();

				if let serde_json::Value::Null = pair.1 {
					query += "NULL, "
				} else if field.ty == PgType::Serial {
					query += "DEFAULT, "
				} else {
					query += "$";
					query += value_num.to_string().as_str();
					query += ", ";
					value_num += 1;
			
					match field.ty {
						PgType::Real => f32s.push(unwrap_num(pair.1).as_f64().unwrap() as f32),
						PgType::DoublePrecision => f64s.push(unwrap_num(pair.1).as_f64().unwrap()),
						PgType::Char => i8s.push(unwrap_num(pair.1).as_i64().unwrap() as i8),
						PgType::SmallInt => i16s.push(unwrap_num(pair.1).as_i64().unwrap() as i16),
						PgType::Integer => i32s.push(unwrap_num(pair.1).as_i64().unwrap() as i32),
						PgType::BigInt => i64s.push(unwrap_num(pair.1).as_i64().unwrap()),
						PgType::IpAddr => addrs.push(serde_json::from_value(pair.1.clone()).unwrap()),
						PgType::TimeStamp => tss.push(serde_json::from_value(pair.1.clone()).unwrap()),
						_ => {},
					};
				}
			}
			query.pop();
			query.pop();
			query += "),";
		}
		query.pop();
		query += format!(" RETURNING {}", scheme.pk_field.unwrap().name).as_str();

		let mut j_values = vec!();
		let mut j_maps = vec!();
		for i in 0..items.len() {
			let val = serde_json::to_value(&items[i]).unwrap();
			j_values.push(val);
		}
		for i in 0..j_values.len() {
			let map = j_values[i].as_object().unwrap();
			j_maps.push(map);
		}

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();

		let mut ts_iter = tss.iter();
		let mut addr_iter = addrs.iter();

		for map in j_maps {
			for pair in map {
				let opt_field = scheme.fields.get(pair.0);
				if let None = opt_field {
					continue;
				}
				let field = opt_field.unwrap();

				if let serde_json::Value::Null = pair.1 {
					continue;
				}
				if field.ty == PgType::Serial {
					continue;
				}

				match field.ty {
					PgType::Real => values.push(f32_iter.next().unwrap()),
					PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
					PgType::Char => values.push(i8_iter.next().unwrap()),
					PgType::SmallInt => values.push(i16_iter.next().unwrap()),
					PgType::Integer => values.push(i32_iter.next().unwrap()),
					PgType::BigInt => values.push(i64_iter.next().unwrap()),
					PgType::TimeStamp => values.push(ts_iter.next().unwrap()),
					PgType::IpAddr => values.push(addr_iter.next().unwrap()),
					PgType::Boolean => {
						if let serde_json::Value::Bool(b) = pair.1 {
							values.push(b);
						} else {
							panic!("Expected bool, found {}", pair.1);
						}
					},
					PgType::Text => {
						if let serde_json::Value::String(s) = pair.1 {
							values.push(s);
						} else {
							panic!("Expected string, found {}", pair.1);
						}
					},
					_ => {},
				};
			}
		}
	
		println!("{}", query);
		println!("{:?}", values);

		let rows = self.client.query(query.as_str(), values.as_slice())?;

		if rows.len() == 0 {
			Result::Err(DbError::DataError(DataError::ZeroRecordReturned))
		} else if rows.len() != items.len() {
			Result::Err(DbError::DataError(DataError::WrongNumberOfRecordsReturned(items.len(), rows.len())))
		} else {
			let mut i = 0;
			for item in items.iter_mut() {
				let id: P = rows[i].get(0);
				item.__set_pk(id);
				i += 1;
			}
			Ok(())
		}
	}

	pub fn select_all<'de, T>(&mut self) -> Result<Vec<T>, DbError>
	where T: Entity + Serialize + Deserialize<'de> {

		let scheme = T::scheme();
		let query = format!("SELECT * FROM {}", scheme.name);
		let rows = self.client.query(query.as_str(), &[])?;
		let mut res: Vec<T> = vec!();
		for row in rows {
			res.push(from_row(row)?);
		}
		Ok(res)
	}

	pub fn select_by_pk<'a, 'de, P, T>(&mut self, v: P) -> Result<T, DbError>
	where P: ToSql + FromSql<'a> + std::marker::Sync,
	      T: Entity + WithId<'a, P> + Serialize + Deserialize<'de> {

		let values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!(&v);
		let scheme = T::scheme();
		let query = format!(r#"SELECT * FROM {}
							WHERE {} = $1"#, scheme.name, scheme.pk_field.unwrap().name);
		let mut rows = self.client.query(query.as_str(), values.as_slice())?;
		if rows.len() == 0 {
			Result::Err(DbError::DataError(DataError::ZeroRecordReturned))
		} else if rows.len() > 1 {
			Result::Err(DbError::DataError(DataError::MoreThan1RecordReturned))
		} else {
			Ok(from_row(rows.remove(0))?)
		}
	}

	pub fn delete_by_pk<'a, P, T>(&mut self, v: P) -> Result<u64, DbError>
	where P: ToSql + FromSql<'a> + std::marker::Sync,
	      T: Entity + WithId<'a, P> {

		let values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!(&v);
		let scheme = T::scheme();
		let query = format!(r#"DELETE FROM {}
							WHERE {} = $1"#, scheme.name, scheme.pk_field.unwrap().name);
		let rows_affected = self.client.execute(query.as_str(), values.as_slice())?;
		Ok(rows_affected)
	}

	pub fn count<T: Entity>(&mut self) -> Result<i64, DbError> {
		let scheme = T::scheme();
		let query = format!("SELECT COUNT(*) FROM {}", scheme.name);
		let rows = self.client.query(query.as_str(), &[])?;
		Ok(rows[0].get(0))
	}

	pub fn update<'a, P, T>(&mut self, item: &mut T) -> Result<u64, DbError>
	where P: ToSql + FromSql<'a> + std::marker::Sync,
	      T: Entity + WithId<'a, P> + Serialize {

		let scheme = T::scheme();
		let val = serde_json::to_value(&item).unwrap();
		let pk_name = scheme.pk_field.unwrap().name;
	
		println!("{}", val);
		let mut query = String::new();

		//TODO: error handling
		let map = val.as_object().unwrap();
	
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();

		let mut tss: Vec<SystemTime> = vec!();
		let mut addrs: Vec<IpAddr> = vec!();
		
		for pair in map {
			if pair.0.as_str() == pk_name {
				continue;
			}

			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			query += pair.0.as_str();
			query += " = ";

			if let serde_json::Value::Null = pair.1 {
				query += "NULL, "
			} else if field.ty == PgType::Serial {
				query += "DEFAULT, "
			} else {
				query += "$";
				query += value_num.to_string().as_str();
				query += ", ";
				value_num += 1;
		
				match field.ty {
					PgType::Real => f32s.push(unwrap_num(pair.1).as_f64().unwrap() as f32),
					PgType::DoublePrecision => f64s.push(unwrap_num(pair.1).as_f64().unwrap()),
					PgType::Char => i8s.push(unwrap_num(pair.1).as_i64().unwrap() as i8),
					PgType::SmallInt => i16s.push(unwrap_num(pair.1).as_i64().unwrap() as i16),
					PgType::Integer => i32s.push(unwrap_num(pair.1).as_i64().unwrap() as i32),
					PgType::BigInt => i64s.push(unwrap_num(pair.1).as_i64().unwrap()),
					PgType::IpAddr => addrs.push(serde_json::from_value(pair.1.clone()).unwrap()),
					PgType::TimeStamp => tss.push(serde_json::from_value(pair.1.clone()).unwrap()),
					_ => {},
				};
			}
		}
		query.pop();
		query.pop();

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();

		let mut ts_iter = tss.iter();
		let mut addr_iter = addrs.iter();
	
		for pair in map {
			if pair.0.as_str() == pk_name {
				continue;
			}
		
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				continue;
			}
			if field.ty == PgType::Serial {
				continue;
			}

			match field.ty {
				PgType::Real => values.push(f32_iter.next().unwrap()),
				PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
				PgType::Char => values.push(i8_iter.next().unwrap()),
				PgType::SmallInt => values.push(i16_iter.next().unwrap()),
				PgType::Integer => values.push(i32_iter.next().unwrap()),
				PgType::BigInt => values.push(i64_iter.next().unwrap()),
				PgType::TimeStamp => values.push(ts_iter.next().unwrap()),
				PgType::IpAddr => values.push(addr_iter.next().unwrap()),
				PgType::Boolean => {
					if let serde_json::Value::Bool(b) = pair.1 {
						values.push(b);
					} else {
						panic!("Expected bool, found {}", pair.1);
					}
				},
				PgType::Text => {
					if let serde_json::Value::String(s) = pair.1 {
						values.push(s);
					} else {
						panic!("Expected string, found {}", pair.1);
					}
				},
				_ => {},
			};
		}
	
		println!("{}", query);
		println!("{:?}", values);

		query = format!("UPDATE {} SET {} WHERE {} = ${}", scheme.name, query, pk_name, value_num.to_string());
		values.push(item.__get_pk());
	
		let rows_affected = self.client.execute(query.as_str(), values.as_slice())?;
		Ok(rows_affected)
	}

	pub fn delete_full_match<T: Entity + Serialize>(&mut self, item: &T) -> Result<u64, DbError> {
		
		let scheme = T::scheme();
		let val = serde_json::to_value(item).unwrap();
	
		println!("{}", val);
		let mut query = format!("DELETE FROM {} WHERE ", scheme.name);

		//TODO: error handling
		let map = val.as_object().unwrap();
		
		let mut value_num = 1;

		let mut f32s: Vec<f32> = vec!();
		let mut f64s: Vec<f64> = vec!();

		let mut i8s: Vec<i8> = vec!();
		let mut i16s: Vec<i16> = vec!();
		let mut i32s: Vec<i32> = vec!();
		let mut i64s: Vec<i64> = vec!();

		let mut tss: Vec<SystemTime> = vec!();
		let mut addrs: Vec<IpAddr> = vec!();
		
		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			query += pair.0.as_str();

			if let serde_json::Value::Null = pair.1 {
				query += "IS NULL"
			} else {
				query += " = $";
				query += value_num.to_string().as_str();
				value_num += 1;
		
				match field.ty {
					PgType::Real => f32s.push(unwrap_num(pair.1).as_f64().unwrap() as f32),
					PgType::DoublePrecision => f64s.push(unwrap_num(pair.1).as_f64().unwrap()),
					PgType::Char => i8s.push(unwrap_num(pair.1).as_i64().unwrap() as i8),
					PgType::SmallInt => i16s.push(unwrap_num(pair.1).as_i64().unwrap() as i16),
					PgType::Integer => i32s.push(unwrap_num(pair.1).as_i64().unwrap() as i32),
					PgType::BigInt => i64s.push(unwrap_num(pair.1).as_i64().unwrap()),
					PgType::IpAddr => addrs.push(serde_json::from_value(pair.1.clone()).unwrap()),
					PgType::TimeStamp => tss.push(serde_json::from_value(pair.1.clone()).unwrap()),
					_ => {},
				};
			}
			query += " AND ";
		}
		query.pop();
		query.pop();
		query.pop();
		query.pop();
		query.pop();

		let mut values: Vec<&(dyn postgres::types::ToSql + Sync)> = vec!();

		let mut f32_iter = f32s.iter();
		let mut f64_iter = f64s.iter();

		let mut i8_iter = i8s.iter();
		let mut i16_iter = i16s.iter();
		let mut i32_iter = i32s.iter();
		let mut i64_iter = i64s.iter();

		let mut ts_iter = tss.iter();
		let mut addr_iter = addrs.iter();
	
		for pair in map {
			let opt_field = scheme.fields.get(pair.0);
			if let None = opt_field {
				continue;
			}
			let field = opt_field.unwrap();

			if let serde_json::Value::Null = pair.1 {
				continue;
			}
			if field.ty == PgType::Serial {
				continue;
			}

			match field.ty {
				PgType::Real => values.push(f32_iter.next().unwrap()),
				PgType::DoublePrecision => values.push(f64_iter.next().unwrap()),
				PgType::Char => values.push(i8_iter.next().unwrap()),
				PgType::SmallInt => values.push(i16_iter.next().unwrap()),
				PgType::Integer => values.push(i32_iter.next().unwrap()),
				PgType::BigInt => values.push(i64_iter.next().unwrap()),
				PgType::TimeStamp => values.push(ts_iter.next().unwrap()),
				PgType::IpAddr => values.push(addr_iter.next().unwrap()),
				PgType::Boolean => {
					if let serde_json::Value::Bool(b) = pair.1 {
						values.push(b);
					} else {
						panic!("Expected bool, found {}", pair.1);
					}
				},
				PgType::Text => {
					if let serde_json::Value::String(s) = pair.1 {
						values.push(s);
					} else {
						panic!("Expected string, found {}", pair.1);
					}
				},
				_ => {},
			};
		}
	
		println!("{}", query);
		println!("{:?}", values);
	
		let rows_affected = self.client.execute(query.as_str(), values.as_slice())?;
		Ok(rows_affected)
	}
}

fn unwrap_num(val: &serde_json::value::Value) -> &serde_json::Number {
	if let serde_json::Value::Number(num) = val {
		num
	} else {
		panic!("Number expected, found {}", val);
	}
}