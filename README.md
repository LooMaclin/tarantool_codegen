#Tarantool ORM

#Add dependency

```toml



```

#Usage

```rust
#[macro_use]
extern crate tarantool_orm;
extern crate rmpv;
extern crate tarantool;

use tarantool::{Value, SyncClient, IteratorType, Select, Insert, Replace, Delete, UpdateCommon,
                CommonOperation, Call, Eval, UpdateString, UpdateInteger, IntegerOperation, Upsert,
                UpsertOperation, Space, Utf8String, ToMsgPack};
use std::fmt::Debug;
```