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
