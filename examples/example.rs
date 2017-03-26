#![deny(warnings, bad_style, unused, future_incompatible)]

extern crate rmpv;
extern crate tarantool;

#[macro_use]
extern crate tarantool_codegen;


use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Delete, Eval, ToMsgPack};

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
    println!("Example 1. Insert single object in space: ");
    println!("===============");

    println!("Insert user into User space result: {:?}", User {
        id: 0,
        login: String::from("test_insert_single_object"),
        password: String::from("test_insert_single_object"),
        likes: 1,
        posts: 1,
    }.insert(&mut tarantool_instance));

    println!("===============");
    println!("Insert users: ");
    println!("===============");
    for (index, insert_user_result) in User::insert_group(
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


    println!("===============");
    println!("Clearing space after examples...");
    println!("===============");
    for (index, user) in User::select(Select {
        space: 512,
        index: 0,
        limit: 10,
        offset: 0,
        keys: vec![],
        iterator: IteratorType::All,
    }, &mut tarantool_instance).into_iter().enumerate() {
        println!("Deleting user №{}... : {:?}", index, user.delete(&mut tarantool_instance));
    }
    println!("Space is clear.");
}