use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Constraint {
	PrimaryKey,
	References(String, String),
	Unique,
	NotNull,
	Null,
}

impl ToString for Constraint {
	fn to_string(&self) -> String {
		match self {
			Constraint::PrimaryKey => "PRIMARY KEY".to_string(),
			Constraint::References(table, column) => format!("REFERENCES {}({})", table, column),
			Constraint::Unique => "UNIQUE".to_string(),
			Constraint::NotNull => "NOT NULL".to_string(),
			Constraint::Null => "NULL".to_string()
		}
	}
}

type Serial = i32;

#[derive(Deserialize, Serialize)]
pub enum PgType {
	Serial,
	Real,
	DoublePrecision,
	Text,
	Char,
	SmallInt,
	Integer,
	BigInt,
	Boolean,
	ByteArray,
	TimeStamp,
	IpAddr,
}

pub fn pg_type_to_str(ty: &PgType) -> &'static str {
	match ty {
		PgType::Serial => "serial",
		PgType::Real => "real",
		PgType::DoublePrecision => "double precision",
		PgType::Text => "text",
		PgType::Char => "char",
		PgType::SmallInt => "smallint",
		PgType::Integer => "integer",
		PgType::BigInt => "bigint",
		PgType::Boolean => "bool",
		PgType::ByteArray => "bytea",
		PgType::TimeStamp => "timestamp",
		PgType::IpAddr => "INET"
	}
}

#[derive(Deserialize, Serialize)]
pub struct Field {
	pub ty: PgType,
	pub constraints: Vec<Constraint>
}

#[derive(Deserialize, Serialize)]
pub struct Scheme {
	pub name: String,
	pub fields: HashMap<String, Field>
}