//! This crate allows you to serialize and deserialize any type that implements
//! `serde::Serialize` and `serde::Deserialzie` into/from `rlua::Value`.
//!
//! Implementation is similar to `serde_json::Value`
//!
//! Example usage:
//!
//! ```rust
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate rlua;
//! extern crate rlua_serde;
//!
//! fn main() {
//!     #[derive(Serialize, Deserialize)]
//!     struct Foo {
//!         bar: u32,
//!         baz: Vec<String>,
//!     }
//!
//!     let lua = rlua::Lua::new();
//!     let foo = Foo {
//!         bar: 42,
//!         baz: vec![String::from("fizz"), String::from("buzz")],
//!     };
//!
//!     let value = rlua_serde::to_value(&lua, &foo).unwrap();
//!     lua.globals().set("value", value).unwrap();
//!     lua.exec::<_, ()>(
//!         r#"
//!             assert(value["bar"] == 42)
//!             assert(value["baz"][2] == "buzz")
//!         "#,
//!         None,
//!     ).unwrap()
//! }
//! ```

extern crate rlua;
#[macro_use]
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod ser;
pub mod de;


use rlua::{Lua, Value, Error};


pub fn to_value<T: serde::Serialize>(lua: &Lua, t: T) -> Result<Value, Error> {
    let serializer = ser::Serializer { lua };
    Ok(t.serialize(serializer)?)
}


pub fn from_value<'de, T: serde::Deserialize<'de>>(value: Value<'de>) -> Result<T, Error> {
    let deserializer = de::Deserializer { value };
    Ok(T::deserialize(deserializer)?)
}
