#![crate_type = "proc-macro"]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
extern crate rmpv;
#[macro_use]
extern crate quote;
extern crate tarantool;

use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
                CommonOperation, Call, Eval, UpdateString, UpdateInteger, IntegerOperation, Upsert,
                UpsertOperation, Space, ToMsgPack};

use proc_macro::TokenStream;
use syn::Ident;
use std::fmt::Debug;

#[proc_macro_derive(Space)]
pub fn derive_space(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = new_space(ast);

    result.to_string().parse().expect("couldn't parse string to tokens")
}

#[proc_macro_derive(ToMsgPack)]
pub fn derive_to_msg_pack(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = new_to_msg_pack(ast);

    result.to_string().parse().expect("couldn't parse string to tokens")
}

fn new_to_msg_pack(ast: syn::MacroInput) -> quote::Tokens {
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let name = &ast.ident;
            let field_idents: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
            quote! {
                impl ToMsgPack for #name {
                    fn get_msgpack_representation(&self) -> Vec<Value> {
                        let mut result : Vec<Value> = Vec::new();
                        #(
                            result.push(Value::from(self.#field_idents.clone()));
                        )*
                        result
                    }
                }
            }
        },
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}

fn new_space(ast: syn::MacroInput) -> quote::Tokens {
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let name = &ast.ident;
            let name_string = name.as_ref();
            let field_idents_msgpack: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
            let field_idents_select = field_idents_msgpack.clone();
            let mut field_numbers = Vec::new();
            for (number, field) in field_idents_select.iter().enumerate() {
                field_numbers.push(number);
            }
            let mut tarantool_instance = SyncClient::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
                panic!("err: {}", err);
            });

            let error_handler = |err| panic!("Tarantool error: {}", err);
            let space_id = match tarantool_instance.fetch_space_id(ast.ident.as_ref()) {
                Ok(space_id) => {
                    println!("Space with name {} exist with id {}", ast.ident.as_ref(), space_id);
                    space_id
                },
                Err(err) => {
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

                    fn insert(data: Vec<#name>, connection: &mut SyncClient) -> Vec<Result<Value, Utf8String>> {
                        data.into_iter().map(|element| {
                        let max_index = connection.request(&Eval {
                                expression: format!("return box.space.{}.index.primary:max()", #name_string).into(),
                                keys: vec![]
                            }).unwrap()[0][0].as_u64().unwrap();
                            let mut msg_pack_repr = element.get_msgpack_representation();
                            msg_pack_repr.insert(0, Value::from(max_index+1));
                            connection.request(&Insert {
                                space: #space_id,
                                keys: msg_pack_repr,
                            })
                        })
                        .collect::<Vec<Result<Value, Utf8String>>>()
                    }

                    fn select(select_params: &Select, connection: &mut SyncClient) -> Vec<#name> {
                            connection.request(select_params)
                                .unwrap()
                                .as_array().unwrap().into_iter().map(|element| {
                                   #name { #(
                                      #field_idents_select : element[#field_numbers],
                                   )* }
                                })
                                .collect::<Vec<User>>()
                    }
                }
            }
        },
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}