extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashMap;
use proc_macro::TokenStream;
use rs_pg_scheme::{Constraint, Field, Scheme, PgType};

const TABLE_NAME_ATTR: &'static str = "table_name";
const UNIQUE_ATTR: &'static str = "unique";
const PRIMARY_KEY_ATTR: &'static str = "primary_key";
const REFERENCES_ATTR: &'static str = "references";
const SKIP_ATTR: &'static str = "skip";
const CHECK_ATTR: &'static str = "check";

#[proc_macro_derive(Entity, attributes(table_name, primary_key, references, unique, serial, skip, check))]
pub fn entity(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_entity(&ast);
    
    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_entity(ast: &syn::DeriveInput) -> quote::Tokens {

	let type_name = &ast.ident;	

	let mut fields_map = HashMap::new();

	let fields = get_fields(ast);	
	for field in fields {
		if let Some(_) = &field.attrs.iter().find(|a| a.name() == SKIP_ATTR) {
			continue;
		}

		let mut constr = vec!();

		let (field_type, is_nullable) = get_field_type(&field);
		constr.push(
			if is_nullable {
				Constraint::Null
			} else {
				Constraint::NotNull
			}
		);

		
		if let Some(_) = &field.attrs.iter().find(|a| a.name() == UNIQUE_ATTR) {
			constr.push(Constraint::Unique);
		}
		
		if let Some(_) = &field.attrs.iter().find(|a| a.name() == PRIMARY_KEY_ATTR) {
			constr.push(Constraint::PrimaryKey);
		}

		if let Some(attr) = &field.attrs.iter().find(|a| a.name() == REFERENCES_ATTR) {

			if let syn::MetaItem::List(_, ref nested) = attr.value {
				if nested.len() != 2 {
					panic!("Argument mismatch. Expected 2 arguments: table and column in references attribute, {} provided", nested.len());	
				}

				let table_ref = unwrap_reference(&nested[0]);
				let column_ref = unwrap_reference(&nested[1]);
				constr.push(Constraint::References(table_ref, column_ref));
			}
		}

		fields_map.insert(
			get_field_name(&field), 
			Field{
				ty: field_type,
				constraints: constr,
			}
		);
	}

	let scheme = Scheme{
		name: get_table_name(ast),
		fields: fields_map,
	};

	let j = serde_json::to_string(&scheme).unwrap();
	println!("{}", j);
	let json_scheme = j.as_str();

    quote! {
        impl Entity for #type_name {
            fn scheme() -> Scheme {
				let scheme: Scheme = serde_json::from_str(#json_scheme).unwrap();
				scheme
			}
		}
    }
}

fn get_table_name(ast: &syn::DeriveInput) -> String {
	if let Some(ref a) = &ast.attrs.iter().find(|a| a.name() == TABLE_NAME_ATTR) {
		if let syn::MetaItem::NameValue(_, ref nested) = a.value {
			if let syn::Lit::Str(name, _) = nested {
				name.clone()
			} else {
				panic!("Table name should be defined with string");
			}
		} else {
			panic!("Table name used and not defined");
		}
	} else {
		format!("{}", &ast.ident)
	}
}

fn get_fields(ast: &syn::DeriveInput) -> &Vec<syn::Field> {
	if let syn::Body::Struct(variant_data) = &ast.body {
		if let syn::VariantData::Struct(fields) = variant_data {
			fields
		} else {
			panic!("Only struct can be entity");
		}
	} else {
		panic!("Only struct can be entity");
	}
}

fn get_field_name(field: &syn::Field) -> String {
	if let Some(ident) = &field.ident {
		format!("{}", ident)
	} else {
		panic!("Fields should have names");
	}
}

fn get_field_type(field: &syn::Field) -> (PgType, bool) {

	if let syn::Ty::Path(_, path) = &field.ty {
		let last_segment = path.segments.last().unwrap();
		let str_type = format!("{}", last_segment.ident);

		if str_type == "Option" {
			if let syn::PathParameters::AngleBracketed(angle_params) = &last_segment.parameters {
				if let syn::Ty::Path(_, inner_path) = &angle_params.types[0] {
					let inner_type = format!("{}", inner_path.segments.last().unwrap().ident);
					(match_type(inner_type), true)
				} else { 
					panic!("Unsupprted Option inner type {:?}", angle_params);
				}
			} else {
				panic!("Only angle parameters for Option is supported");
			}
		} else {
			(match_type(str_type), false)
		}
	} else {
		panic!("Unsupported field type: {:?}", field.ty);
	}
}

fn match_type(rust_type: String) -> PgType {

	match rust_type.as_str() {
		"f32" => PgType::Real,
		"f64" => PgType::DoublePrecision,
		"String" => PgType::Text,
		"i8" => PgType::Char,
		"i16" => PgType::SmallInt,
		"i32" => PgType::Integer,
		"i64" => PgType::BigInt,
		"bool" => PgType::Boolean,
		"Serial" => PgType::Serial,
		"SystemTime" => PgType::TimeStamp,
		"IpAddr" => PgType::IpAddr,
		_ => panic!("Unsupported rust type"),
	}
}

fn unwrap_reference(meta_item: &syn::NestedMetaItem) -> String {
	if let syn::NestedMetaItem::Literal(lit) = meta_item {
		if let syn::Lit::Str(value, _) = lit {
			value.to_string()
		} else {
			panic!("Expected table and column in references attribute");
		}
	} else {
		panic!("Table name should be defined with string");
	}
}