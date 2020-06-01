extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

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

	const TABLE_NAME_ATTR: &'static str = "table_name";
	const UNIQUE_ATTR: &'static str = "unique";
	const PRIMARY_KEY_ATTR: &'static str = "primary_key";
	const REFERENCES_ATTR: &'static str = "references";
	const SKIP_ATTR: &'static str = "skip";
	const CHECK_ATTR: &'static str = "check";

	let type_name = &ast.ident;

	let table_name = 
		if let Some(ref a) = ast.attrs.iter().find(|a| a.name() == TABLE_NAME_ATTR) {
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
			format!("{}", type_name)
		};

	let fields =
		if let syn::Body::Struct(variant_data) = &ast.body {
			if let syn::VariantData::Struct(fields) = variant_data {
				fields
			} else {
				panic!("Only struct can be entity");
			}
		} else {
			panic!("Only struct can be entity1");
		};

	let mut table_scheme = table_name + "(";
	
	for field in fields {
		if let Some(_) = field.attrs.iter().find(|a| a.name() == SKIP_ATTR) {
			continue;
		}

		let field_name =
			if let Some(ident) = &field.ident {
				format!("{}", ident)
			} else {
				panic!("Fields should have names");
			};

		table_scheme += field_name.as_str();
		table_scheme += " ";

		let field_type =
			if let syn::Ty::Path(_, path) = &field.ty {
				format!("{}", path.segments.last().unwrap().ident)
			} else {
				panic!("Unsupported field type: {:?}", field.ty);
			};
		
		table_scheme += match_type(field_type);
		table_scheme += " ";

		if let Some(_) = field.attrs.iter().find(|a| a.name() == UNIQUE_ATTR) {
			table_scheme += "UNIQUE "
		}
		
		if let Some(_) = field.attrs.iter().find(|a| a.name() == PRIMARY_KEY_ATTR) {
			table_scheme += "PRIMARY KEY "
		}

		if let Some(attr) = field.attrs.iter().find(|a| a.name() == REFERENCES_ATTR) {
			table_scheme += "REFERENCES ";
			println!("Referenced {:?}", attr);
			if let syn::MetaItem::List(_, ref nested) = attr.value {
				if nested.len() != 2 {
					panic!("Argument mismatch. Expected table and column in references attribute");	
				}

				if let syn::NestedMetaItem::Literal(lit) = &nested[0] {
					if let syn::Lit::Str(ref_table, _) = lit {
						table_scheme += ref_table.as_str();
					} else {
						panic!("Expected table and column in references attribute");
					}
				} else {
					panic!("Table name should be defined with string");
				}

				table_scheme += "(";

				if let syn::NestedMetaItem::Literal(lit) = &nested[1] {
					if let syn::Lit::Str(ref_col, _) = lit {
						table_scheme += ref_col.as_str();
					} else {
						panic!("Expected table and column in references attribute");
					}
				} else {
					panic!("Table name should be defined with string");
				}

				table_scheme += ")";

			}
		}

		table_scheme += ",\n ";
	}

	table_scheme.pop();
	table_scheme.pop();
	table_scheme.pop();

	table_scheme += ")";
	println!("{}", table_scheme);

    quote! {
        impl Entity for #type_name {
            fn scheme() -> String {
                String::from(#table_scheme)
            }
        }
    }
}

fn match_type(rust_type: String) -> &'static str {

	match rust_type.as_str() {
		"f32" => "real",
		"f64" => "double precision",
		"String" => "text",
		"i32" => "integer",
		"bool" => "boolean",
		_ => panic!("Unsupported rust type"),
	}
}

enum Constraint {
	PrimaryKey,
	References,
	Unique,

}

enum PgType {
	Serial,
	Real,
	DoublePrecision,
	Text,
	Integer,
	Boolean,
}

struct Field {
	name: String,
	ty: PgType,
	constraints: Vec<Constraint>
}

struct Scheme {
	name: String,
	fields: Vec<Field>
}