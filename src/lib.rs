// #![feature(const_type_id)]

// extern crate type_info;

// #[macro_use]
// extern crate type_info_derive;

// use type_info::{
// 	TypeInfo,
// 	DynamicTypeInfo,
// 	Type,
// 	TypeId,
// 	Data,
// };

pub mod database;

// trait Entity {
// 	const TABLE_NAME: String;
// 	fn fields() -> Vec<String>;
// 	//const TYPE: Type;
// }

// impl Ttt for String {
// 	fn isT(&self) -> bool {
// 		self.len() > 0
// 	}
// 	const TYPE: Type = Type {
// 		id: TypeId::of::<String>(
// 		),
// 		module: "",
// 		ident: stringify!(String),
// 		data: Data::Primitive,
// 	};
// }

// impl DynamicTypeInfo for String {
// 	fn type_ref(&self) -> &'static Type {
// 		&<Self as TypeInfo>::TYPE
// 	}
// }

// //#[derive(Entity)]
// struct Person {
//     name: String,
//     age: u32,
// }

// pub fn test() {
// 	let ty = Person::TYPE;

//     assert_eq!("Person", ty.ident);
//     assert_eq!(vec!["name", "age"], ty.fields().iter().map(|f| f.ident.unwrap()).collect::<Vec<_>>());
// }



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
		assert_eq!(2 + 2, 4);
    }
}