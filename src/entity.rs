use serde::{Serialize, Deserialize};

pub trait Entity {//<'a> : TypeName + Serialize + Deserialize<'a> {
    fn scheme() -> String;
}

macro_rules! type_name {
    ($e:expr) => { Attribute::process(&$e) };
}