use rs_pg_scheme::Scheme;
use postgres::types::{
	ToSql,
	FromSql
};

pub trait Entity {
	fn scheme() -> Scheme;
}

pub trait WithId<'a, T: ToSql + FromSql<'a>> {
	fn __get_pk(&self) -> &T;
	fn __set_pk(&mut self, v: T);
	fn __borrow_pk(&self) -> T;

//	fn __assign_id_from_
}