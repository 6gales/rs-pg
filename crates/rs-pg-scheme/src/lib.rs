use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Action {
	Restrict,
	Cascade,
	SetNull,
	SetDefault,
	NoAction
}

impl ToString for Action {
	fn to_string(&self) -> String {
		match self {
			Action::Restrict => "RESTRICT".to_string(),
			Action::Cascade => "CASCADE".to_string(),
			Action::SetNull => "SET NULL".to_string(),
			Action::SetDefault => "SET DEFAULT".to_string(),
			Action::NoAction => "NO ACTION".to_string()
		}
	}
}

#[derive(Deserialize, Serialize)]
pub enum Constraint {
	PrimaryKey,
	References(String, String, Option<Action>, Option<Action>),
	Unique,
	NotNull,
	Null,
	Check(String)
}

impl ToString for Constraint {
	fn to_string(&self) -> String {
		match self {
			Constraint::PrimaryKey => "PRIMARY KEY".to_string(),
			Constraint::References(table, column, delete, update) => {
				format!("REFERENCES {}({}) ON DELETE {} ON UPDATE {}", table, column,
				if let Some(a) = delete { a.to_string() } else { Action::NoAction.to_string() },
				if let Some(a) = update { a.to_string() } else { Action::NoAction.to_string() })
			}
			Constraint::Unique => "UNIQUE".to_string(),
			Constraint::NotNull => "NOT NULL".to_string(),
			Constraint::Null => "NULL".to_string(),
			Constraint::Check(body) => format!("CHECK ({})", body)
		}
	}
}

pub type Serial = i32;

#[derive(Deserialize, Serialize, PartialEq, Clone)]
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
	Date,
	Time
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
		PgType::IpAddr => "INET",
		PgType::Date => "DATE",
		PgType::Time => "TIME"
	}
}

#[derive(Deserialize, Serialize)]
pub struct Field {
	pub ty: PgType,
	pub constraints: Vec<Constraint>
}

#[derive(Deserialize, Serialize)]
pub struct PkField {
	pub name: String,
	pub ty: PgType
}

#[derive(Deserialize, Serialize)]
pub struct Scheme {
	pub name: String,
	pub pk_field: Option<PkField>,
	pub fields: HashMap<String, Field>,
	pub constraints: Vec<Constraint>
}