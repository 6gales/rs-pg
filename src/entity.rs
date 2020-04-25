use serde::{Serialize, Deserialize};

pub trait TypeName {
	fn name(&self) -> String;
}

pub trait Entity : TypeName + Serialize + Deserialize {
	
}

macro_rules! attribute {
    ($e:expr) => { Attribute::process(&$e) };
}