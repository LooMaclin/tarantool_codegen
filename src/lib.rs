#![crate_type = "proc-macro"]
#![recursion_limit="256"]
// #![deny(warnings, bad_style, unused, future_incompatible)]

extern crate proc_macro;
extern crate syn;
extern crate rmpv;
#[macro_use]
extern crate quote;
extern crate tarantool;

use tarantool::{SyncClient, Eval};

use proc_macro::TokenStream;
use syn::{Ident, Ty};

#[proc_macro_derive(Space)]
pub fn derive_space(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = new_space(ast);

    result.to_string().parse().expect("couldn't parse string to tokens")
}

#[proc_macro_derive(Rest)]
pub fn derive_rest(input: TokenStream) -> TokenStream {
    let input : String = input.to_string();
    let ast = syn::parse_macro_input(&input).expect("couldn't parse item");
    let result = new_rest(ast);
    result.to_string().parse().expect("couldn't parse string to tokens")
}

fn new_rest(ast: syn::MacroInput) -> quote::Tokens {

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(_)) => {
            quote! {

            }
        },
        _ => panic!("#[derive(Rest)] can only be used with structs")
    }
}

fn new_space(ast: syn::MacroInput) -> quote::Tokens {
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let name = &ast.ident;
            let name_string = name.as_ref();
            let field_idents_msgpack: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
            let primary_index_ident = field_idents_msgpack[0].clone();
            let field_idents_select = field_idents_msgpack.clone();
            let field_idents_delete = field_idents_msgpack.clone();
            let mut field_numbers = Vec::new();
            let mut field_initialize = Vec::new();
            for (_, field) in fields.iter().enumerate() {
                if let Ty::Path(_, ref path) = field.ty {
                    match path.segments[0].ident.as_ref() {
                        "String" => {
                            field_initialize.push(quote! {
                                .as_str().unwrap().to_string(),
                            });
                        },
                        "u64" => {
                            field_initialize.push(quote! {
                                .as_u64().unwrap(),
                            });
                        },
                        _ => {
                            panic!("Tarantool Codegen Error: this field type not supported.");
                        }
                    }
                } else {
                    panic!("Only path-types was supported!");
                }
            }
            let field_delete_initialize = field_initialize.clone();
            for (number, _) in field_idents_select.iter().enumerate() {
                field_numbers.push(number);

            }
            let field_delete_numbers = field_numbers.clone();
            let mut tarantool_instance = SyncClient::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
                panic!("err: {}", err);
            });

            let error_handler = |err| panic!("Tarantool error: {}", err);
            let space_id = match tarantool_instance.fetch_space_id(ast.ident.as_ref()) {
                Ok(space_id) => {
                    println!("Space with name {} exist with id {}", ast.ident.as_ref(), space_id);
                    space_id
                },
                Err(_) => {
                    let eval = Eval {
                        expression: format!(r#"box.schema.space.create('{}')"#,ast.ident.as_ref()).into(),
                        keys: vec![],
                    };
                    tarantool_instance.request(&eval).unwrap_or_else(&error_handler);
                    println!("Table with name {} created", ast.ident.as_ref());
                    tarantool_instance.fetch_space_id(ast.ident.as_ref()).unwrap()
                }
            };
            match tarantool_instance.fetch_index_id(space_id, "primary") {
                Ok(index_id) => {
                    println!("Primary index exist: {}", index_id);
                },
                Err(err) => {
                    println!("Primary index not exist: {}", err);
                    println!("Primary tree-index automatically created: {:?}", tarantool_instance.request(&Eval {
                        expression: (format!("box.space.{}", name)+":create_index('primary', { type = 'tree', parts = {1, 'unsigned'} })").into(),
                        keys: vec![],
                    }).unwrap_or_else(&error_handler));
                }
            }

            quote! {
                 impl ToMsgPack for #name {
                    fn get_msgpack_representation(&self) -> Vec<Value> {
                        let mut result : Vec<Value> = Vec::new();
                        #(
                            result.push(Value::from(self.#field_idents_msgpack.clone()));
                        )*
                        result
                    }
                }


                impl #name {

                    fn delete(self, connection: &mut SyncClient) -> Result<#name, String> {
                        match connection.request(&Delete {
                            space: #space_id,
                            index: 0,
                            keys: vec![Value::from(self.#primary_index_ident)],
                        }) {
                            Ok(delete_result) => {
                                Ok(#name { #(
                                      #field_idents_delete : delete_result[0][#field_delete_numbers]#field_delete_initialize
                                   )* })
                            },
                            Err(err) => {
                             println!("delete result: error");
                                Err(err.into_str().unwrap_or(String::from("Tarantool Codegen Error: cannot parse delete error message")))
                            }
                        }
                    }

                    fn get_max_id(connection: &mut SyncClient) -> u64 {
                           connection.request(&Eval {
                                expression: format!("return box.space.{}.index.primary:max()", #name_string).into(),
                                keys: vec![]
                           }).unwrap()[0][0].as_u64().unwrap_or(0)
                    }

                    fn insert_group(data: Vec<#name>, connection: &mut SyncClient) -> Vec<Result<#name, String>> {
                        data.into_iter().map(|mut element| {
                            let max_index = #name::get_max_id(connection);
                            element.id = max_index+1;
                            let msg_pack_repr = element.get_msgpack_representation();
                            match connection.request(&Insert {
                                space: #space_id,
                                keys: msg_pack_repr,
                            }) {
                                Ok(_) => {
                                    Ok(element)
                                },
                                Err(error_string) => {
                                    Err(error_string.into_str().unwrap())
                                }
                            }
                        })
                        .collect::<Vec<Result<#name, String>>>()
                    }

                    fn select(select_params: Select, connection: &mut SyncClient) -> Vec<#name> {
                        let new_select_request = Select {
                            space: #space_id,
                            ..select_params
                        };
                            connection.request(&new_select_request)
                                .unwrap()
                                .as_array()
                                .unwrap_or(&Vec::new())
                                .into_iter()
                                .map(|element| {
                                   #name { #(
                                      #field_idents_select : element[#field_numbers]#field_initialize
                                   )* }
                                })
                                .collect::<Vec<User>>()
                    }
                }
            }
        },
        _ => panic!("#[derive(Space)] can only be used with structs"),
    }
}