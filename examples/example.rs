#[macro_use]
extern crate tarantool_orm;
extern crate rmpv;
extern crate tarantool;

use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
                CommonOperation, Call, Eval, UpdateString, UpdateInteger, IntegerOperation, Upsert,
                UpsertOperation, Space, Utf8String, ToMsgPack};
use std::fmt::Debug;
#[derive(Debug, Space)]
pub struct User {
    pub login: String,
    pub password: String,
    pub likes: u64,
    pub posts: u64,
}

fn main() {
    let mut tarantool_instance = SyncClient::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
        panic!("err: {}", err);
    });

    let error_handler = |err| panic!("Tarantool error: {}", err);

    for result in User::insert(
        vec![
            User {
                login: String::from("loomaclin"),
                password: String::from("123"),
                likes: 1,
                posts: 2,
            },
            User {
                login: String::from("loomaclin1"),
                password: String::from("1234"),
                likes: 5,
                posts: 6,
            }
        ],
        &mut tarantool_instance) {
        println!("Insert result: {:?}", result.unwrap_or_else(&error_handler));
    }
}