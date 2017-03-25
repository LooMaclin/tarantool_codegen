#[macro_use]
extern crate tarantool_orm;
extern crate rmpv;
extern crate tarantool;
extern crate futures;
extern crate hyper;

use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
                CommonOperation, Call, Eval, UpdateString, UpdateInteger, IntegerOperation, Upsert,
                UpsertOperation, Space, Utf8String, ToMsgPack};
use std::fmt::Debug;

use futures::future::FutureResult;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

#[derive(Debug, Space, Rest)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub password: String,
    pub likes: u64,
    pub posts: u64,
}

impl Default for User {
    fn default() -> User {
        User {
            id: 0,
            login: String::from("default_login"),
            password: String::from("default_password"),
            likes: 0,
            posts: 0,
        }
    }
}

fn main() {
    let mut tarantool_instance = SyncClient::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
        panic!("err: {}", err);
    });

//    let error_handler = |err| panic!("Tarantool error: {}", err);
    println!("===============");
    println!("Users in space: ");
    println!("===============");
    for (index, user) in User::select(Select {
                                                space: 512,
                                                index: 0,
                                                limit: 10,
                                                offset: 0,
                                                keys: vec![],
                                                iterator: IteratorType::All,
                                            }, &mut tarantool_instance).into_iter().enumerate() {
        println!("№{}: {:?}", index, user);
        println!("Deleting this user... : {:?}",user.delete(&mut tarantool_instance));
    }
    println!("===============");
    println!("Insert users: ");
    println!("===============");
    for (index, insert_user_result) in User::insert(
                                vec![
                                    User {
                                        id: 0,
                                        login: String::from("loomaclin"),
                                        password: String::from("123"),
                                        likes: 1,
                                        posts: 2,
                                    },
                                    User {
                                        id: 0,
                                        login: String::from("loomaclin1"),
                                        password: String::from("1234"),
                                        likes: 5,
                                        posts: 6,
                                    }
                                ], &mut tarantool_instance).into_iter().enumerate() {
        println!("№{}: {:?}", index, insert_user_result);
    }
    println!("===============");
//
//    for (index, user) in User::select(Select {
//        space: 512,
//        index: 0,
//        limit: 10,
//        offset: 0,
//        keys: vec![],
//        iterator: IteratorType::All,
//    }, &mut tarantool_instance).iter().enumerate() {
//        println!("Selected user {}: {:?}", index, user);
//    }
//    let addr = "127.0.0.1:1337".parse().unwrap();
//    let server = Http::new().bind(&addr, || Ok(User::default())).unwrap();
//    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
//    server.run().unwrap();
}