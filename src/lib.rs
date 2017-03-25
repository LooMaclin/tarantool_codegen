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
use syn::Ty;

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
    let name = &ast.ident;
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            quote! {
                impl Service for #name {
                    type Request = Request;
                    type Response = Response;
                    type Error = hyper::Error;
                    type Future = FutureResult<Response, hyper::Error>;

                    fn call(&self, req: Request) -> Self::Future {
                        futures::future::ok(match (req.method(), req.path()) {
                            (&Get, "/") | (&Get, "/echo") => {
                                Response::new()
                                    .with_header(ContentLength("Hello world".len() as u64))
                                    .with_body("Hello world!")
                            },
                            (&Post, "/echo") => {
                                let mut res = Response::new();
                                if let Some(len) = req.headers().get::<ContentLength>() {
                                    res.headers_mut().set(len.clone());
                                }
                                res.with_body(req.body())
                            },
                            _ => {
                                Response::new()
                                    .with_status(StatusCode::NotFound)
                            }
                        })
                    }

                }
            }
        },
        _ => panic!("#[derive(Rest)] can only be used with structs")
    }
}

fn new_space(ast: syn::MacroInput) -> quote::Tokens {
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let name = &ast.ident;
            let name_string = name.as_ref();
            let field_idents_msgpack: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
            let primary_index_ident = field_idents_msgpack[0].clone();
            let field_idents_select = field_idents_msgpack.clone();
            let field_idents_insert = field_idents_msgpack.clone();
            let field_idents_delete = field_idents_msgpack.clone();
            let mut field_numbers = Vec::new();
            let mut field_initialize = Vec::new();
            for (number, field) in fields.iter().enumerate() {
                println!("FIELD TYPE ({}): {:?}", number, field.ty);
                if let Ty::Path(ref q_self, ref path) = field.ty {
                    println!("Path segment: {:?} ", path.segments[0].ident);
                    match path.segments[0].ident.as_ref() {
                        "String" => {
                            println!("string type");
                            field_initialize.push(quote! {
                                .as_str().unwrap().to_string(),
                            });
                        },
                        "u64" => {
                            println!("u64 type");
                            field_initialize.push(quote! {
                                .as_u64().unwrap(),
                            });
                        },
                        _ => {
                            panic!("fuck off");
                        }
                    }
                } else {
                    panic!("Only path-types was supported!");
                }
            }
            let field_insert_initialize = field_initialize.clone();
            let field_delete_initialize = field_initialize.clone();
            for (number, field) in field_idents_select.iter().enumerate() {
                field_numbers.push(number);

            }
            let field_insert_numbers = field_numbers.clone();
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

                    fn delete(self, connection: &mut SyncClient) -> Result<#name, String> {
                        match connection.request(&Delete {
                            space: #space_id,
                            index: 0,
                            keys: vec![Value::from(self.#primary_index_ident)],
                        }) {
                            Ok(delete_result) => {
                                Ok(#name { #(
                                      #field_idents_delete : delete_result[#field_delete_numbers]#field_delete_initialize
                                   )* })
                            },
                            Err(err) => {
                                Err(err.into_str().unwrap())
                            }
                        }
                    }

                    fn insert(data: Vec<#name>, connection: &mut SyncClient) -> Vec<Result<#name, String>> {
                        data.into_iter().map(|mut element| {
                        let max_index = connection.request(&Eval {
                                expression: format!("return box.space.{}.index.primary:max()", #name_string).into(),
                                keys: vec![]
                            }).unwrap()[0][0].as_u64().unwrap_or(0);
                            element.id = max_index+1;
                            let mut msg_pack_repr = element.get_msgpack_representation();
                            match connection.request(&Insert {
                                space: #space_id,
                                keys: msg_pack_repr,
                            }) {
                                Ok(insert_result) => {
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
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}