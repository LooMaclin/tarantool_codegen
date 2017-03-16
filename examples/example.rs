#[macro_use]
extern crate tarantool_orm;
extern crate rmpv;
extern crate tarantool;

use tarantool::{Value, Insertable};

#[derive(Insertable)]
pub struct User {
    a: u8,
    b: u32
}

fn main() {
    println!("User insetable: {:?}", User {
        a: 1,
        b: 2,
    }.get_msgpack_representation())
}