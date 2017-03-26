# Tarantool Codegen

# Index

- [Add dependency](#add-dependency)
- [Usage](#usage)
- [Space annotation overview](#space-annotation)
- [Creating struct for examples](#creating-struct-for-examples)
- [Atomic actions](#atomic-actions)
    - [Insert](#insert)
    - [Select](#select)
    - [Delete](#delete)
- [Group actions](#group-actions)
    - [Insert group](#insert-group)
    - [Select group](#select-group)
    - [Delete group](#delete-group)

# Add dependency

```toml

tarantool_rs = { git = "https://github.com/LooMaclin/tarantool_rs.git", branch = "master" }
tarantool_codegen = { git = "https://github.com/LooMaclin/tarantool_codegen.git", branch = "master" }

```

# Usage

```rust

extern crate rmpv;
extern crate tarantool;

#[macro_use]
extern crate tarantool_codegen;


use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Delete, Eval, ToMsgPack};

```

## Space annotation

`Space` annotation generates for struct two methods types:

- Atomic actions:

    - `insert`;
    - `delete`;
    - `select`;
    
- Group actions that hide atomic:

    - `insert_group`;
    - `select_group`;
    - `delete_group`;

For structures annotated in this way, a requirement is made, the first field must be necessarily `id: U64`.

## Creating struct for examples

```rust

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

```

## Insert

```rust

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

```

## Insert group

```rust

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

```

## Select

```rust

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
        println!("User №{}... : {:?}", index, user);
    }

```

## Delete

```rust

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

```
