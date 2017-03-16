#[macro_use]
extern crate tarantool_orm;
extern crate rmpv;
extern crate tarantool;

use tarantool::{Value, Tarantool, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
                CommonOperation, Call, Eval, UpdateString, UpdateInteger, IntegerOperation, Upsert,
                UpsertOperation, Insertable};

#[derive(Debug, Insertable)]
pub struct User {
    pub login: String,
    pub password: String,
    pub likes: u64,
    pub posts: u64,
}

fn main() {

    let mut tarantool_instance = Tarantool::auth("127.0.0.1:3301", "test", "test").unwrap_or_else(|err| {
        panic!("err: {}", err);
    });

    let error_handler = |err| panic!("Tarantool error: {}", err);

    let select = Select {
        space: 512,
        index: 0,
        limit: 10000,
        offset: 0,
        iterator: IteratorType::All,
        keys: &vec![]
    };

    let tuples = tarantool_instance.request(&select).unwrap_or_else(&error_handler);


    println!("Select result: ");
    for (index, tuple) in tuples.as_array().unwrap().iter().enumerate() {
        let tuple = tuple.as_array().unwrap();
        println!("{}: {:?}", index, tuple);

        let delete = Delete {
            space: 512,
            index: 0,
            keys: &vec![tuple[0].clone()]
        };

        println!("Delete result: {:?}", tarantool_instance.request(&delete).unwrap_or_else(&error_handler));
    }

    let users = vec![
        User {
            login: "user_1".into(),
            password: "123".into(),
            likes: 11,
            posts: 25
        },
        User {
            login: "user_2".into(),
            password: "12345".into(),
            likes: 22,
            posts: 1
        },
        User {
            login: "user_3".into(),
            password: "1664".into(),
            likes: 0,
            posts: 277
        }
    ];

    for (index, user) in users.iter().enumerate() {
        let mut repr = user.get_msgpack_representation();
        repr.insert(0, Value::from(index));
        let insert = Insert {
            space: 512,
            keys: &repr,
        };
        println!("Insert result: {:?}", tarantool_instance.request(&insert).unwrap_or_else(&error_handler));
    }
}