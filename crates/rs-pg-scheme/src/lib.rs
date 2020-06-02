pub enum Constraint {
	PrimaryKey,
	References,
	Unique,

}

pub enum PgType {
	Serial,
	Real,
	DoublePrecision,
	Text,
	Integer,
	Boolean,
}

pub struct Field {
	name: String,
	ty: PgType,
	constraints: Vec<Constraint>
}

pub struct Scheme {
	name: String,
	fields: Vec<Field>
}