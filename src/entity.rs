use serde::{Serialize, Deserialize};

pub trait TypeName {
	fn name(&self) -> &str;
}

pub trait Entity<'a> : TypeName + Serialize + Deserialize<'a> {
}

macro_rules! type_name {
    ($e:expr) => { Attribute::process(&$e) };
}