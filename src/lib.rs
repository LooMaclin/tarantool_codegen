#![crate_type = "proc-macro"]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
extern crate rmpv;
#[macro_use]
extern crate quote;
extern crate tarantool;

use tarantool::{Value, Tarantool, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
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
            let field_idents: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
            let mut tarantool_instance = Tarantool::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
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
                            result.push(Value::from(self.#field_idents.clone()));
                        )*
                        result
                    }
                }


                impl #name {

                    fn insert(data: Vec<#name>, connection: &mut Tarantool) -> Result<Value, Utf8String> {
                        let mut new_keys : Vec<Value> = Vec::new();
                        for key in data {
                            let mut msg_pack_repr = key.get_msgpack_representation();
                            msg_pack_repr.insert(0, Value::from(12));
                            println!("MSG PACK REPR: {:?}", msg_pack_repr);
                            new_keys.push(Value::Array(msg_pack_repr));

                        }
                        println!("new keys: {:?}", new_keys);
                        connection.request(&Insert {
                          space: #space_id,
                          keys: new_keys,
                        })
                    }
                }
            }
        },
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}