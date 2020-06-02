//! Deserialize postgres rows into a Rust data structure.
use serde::de::{
    self,
    Deserialize,
    Visitor,
    IntoDeserializer,
    value::SeqDeserializer
};

use postgres::Row;

use crate::error::{DeError, Result};

use std::{
	net::IpAddr,
	time::SystemTime
};

/// A structure that deserialize Postgres rows into Rust values.
pub struct Deserializer {
    input: Row,
    index: usize,
}

impl Deserializer {
    /// Create a `Row` deserializer from a `Row`.
    pub fn from_row(input: Row) -> Self {
        Self { index: 0, input }
    }
}

/// Attempt to deserialize from a single `Row`.
pub fn from_row<'a, T: Deserialize<'a>>(input: Row) -> Result<T> {
    let mut deserializer = Deserializer::from_row(input);
    Ok(T::deserialize(&mut deserializer)?)
}

macro_rules! unsupported_type {
    ($($fn_name:ident),*,) => {
        $(
            fn $fn_name<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
                Err(DeError::UnsupportedType)
            }
        )*
    }
}

macro_rules! get_value {
	($this:ident, $v:ident, $fn_call:ident, $ty:ty) => {{
        $v.$fn_call($this.input.try_get::<_, $ty>($this.index)
            .map_err(|e| DeError::InvalidType(format!("{:?}", e)))?)
    }}
}

macro_rules! try_get_optional {
	($this:ident, $v:ident, $ty:ty) => {{
		if let Ok(_) = $this.input.try_get::<_, $ty>($this.index) {
			$v.visit_some($this)
		} else {
			$v.visit_none()
		}
	}};

	($this:ident, $v:ident, $ty:ty, $($types:ty), +) => {{
		if let Ok(_) = $this.input.try_get::<_, $ty>($this.index) {
			$v.visit_some($this)
		} else {
			try_get_optional!($this, $v, $($types),+)
		}
	}};
}

impl<'de, 'b> de::Deserializer<'de> for &'b mut Deserializer {
    type Error = DeError;

    unsupported_type! {
        deserialize_any,
        deserialize_u8,
        deserialize_u16,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_bytes,
		deserialize_unit,
//		deserialize_option,
        deserialize_identifier,
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_bool, bool)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i8, i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i16, i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i32, i32)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_i64, i64)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_u32, u32)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_f32, f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_f64, f64)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_string, String)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        get_value!(self, visitor, visit_byte_buf, Vec<u8>)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {		
		try_get_optional!(self, visitor, i32, i64, String, bool, f32, f64, i8, i16, u32, Vec<u8>, SystemTime, IpAddr)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let raw = self.input.try_get::<_, Vec<u8>>(self.index)
            .map_err(|e| DeError::InvalidType(format!("{:?}", e)))?;

        visitor.visit_seq(SeqDeserializer::new(raw.into_iter()))
    }


    fn deserialize_enum<V: Visitor<'de>>(self,
                                         _: &str,
                                         _: &[&str],
                                         _visitor: V)
        -> Result<V::Value>
    {
        //visitor.visit_enum(self)
        Err(DeError::UnsupportedType)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &str, _: V)
        -> Result<V::Value>
    {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &str, _: V)
        -> Result<V::Value>
    {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, _: V)
        -> Result<V::Value>
    {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self,
                                                 _: &str,
                                                 _: usize,
                                                 _: V)
        -> Result<V::Value>
    {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value> {
        self.deserialize_map(v)
    }
}

impl<'de, 'a> de::MapAccess<'de> for Deserializer {
    type Error = DeError;

    fn next_key_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T)
        -> Result<Option<T::Value>>
    {
        if self.index >= self.input.columns().len() {
            return Ok(None)
        }

        self.input.columns()
            .get(self.index)
            .ok_or(DeError::UnknownField)
            .map(|c| c.name().to_owned().into_deserializer())
            .and_then(|n| seed.deserialize(n).map(Some))

    }

    fn next_value_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T)
        -> Result<T::Value>
    {
        let result = seed.deserialize(&mut *self);
        self.index += 1;
        if let Err(DeError::InvalidType(err)) = result {
            let name = self.input.columns().get(self.index - 1).unwrap().name();
            Err(DeError::InvalidType(format!("{} {}", name, err)))
        } else {
            result
        }
    }
}