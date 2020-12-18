use serde;

use rlua::{Context, String as LuaString, Table, Value};

use error::{Error, Result};
use to_value;

pub struct Serializer<'lua> {
    pub lua: Context<'lua>,
}

impl<'lua> serde::Serializer for Serializer<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    type SerializeSeq = SerializeVec<'lua>;
    type SerializeTuple = SerializeVec<'lua>;
    type SerializeTupleStruct = SerializeVec<'lua>;
    type SerializeTupleVariant = SerializeTupleVariant<'lua>;
    type SerializeMap = SerializeMap<'lua>;
    type SerializeStruct = SerializeMap<'lua>;
    type SerializeStructVariant = SerializeStructVariant<'lua>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value<'lua>> {
        Ok(Value::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Value<'lua>> {
        Ok(Value::Integer(value))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value<'lua>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value<'lua>> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value<'lua>> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value<'lua>> {
        Ok(Value::Number(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value<'lua>> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value<'lua>> {
        Ok(Value::String(self.lua.create_string(value)?))
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Value<'lua>> {
        Ok(Value::Table(
            self.lua.create_sequence_from(value.iter().cloned())?,
        ))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value<'lua>> {
        Ok(Value::Nil)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value<'lua>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value<'lua>> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value<'lua>>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value<'lua>>
    where
        T: ?Sized + serde::Serialize,
    {
        let table = self.lua.create_table()?;
        let variant = self.lua.create_string(variant)?;
        let value = to_value(self.lua, value)?;
        table.set(variant, value)?;
        Ok(Value::Table(table))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value<'lua>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value<'lua>>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        let table = self.lua.create_table()?;
        Ok(SerializeVec {
            lua: self.lua,
            idx: 1,
            table,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let name = self.lua.create_string(variant)?;
        let table = self.lua.create_table()?;
        Ok(SerializeTupleVariant {
            lua: self.lua,
            idx: 1,
            name,
            table,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        let table = self.lua.create_table()?;
        Ok(SerializeMap {
            lua: self.lua,
            next_key: None,
            table,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let name = self.lua.create_string(variant)?;
        let table = self.lua.create_table()?;
        Ok(SerializeStructVariant {
            lua: self.lua,
            name,
            table,
        })
    }
}

pub struct SerializeVec<'lua> {
    lua: Context<'lua>,
    table: Table<'lua>,
    idx: u64,
}

impl<'lua> serde::ser::SerializeSeq for SerializeVec<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.table.set(self.idx, to_value(self.lua, value)?)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Value<'lua>> {
        Ok(Value::Table(self.table))
    }
}

impl<'lua> serde::ser::SerializeTuple for SerializeVec<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'lua>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'lua> serde::ser::SerializeTupleStruct for SerializeVec<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'lua>> {
        serde::ser::SerializeSeq::end(self)
    }
}

pub struct SerializeTupleVariant<'lua> {
    lua: Context<'lua>,
    name: LuaString<'lua>,
    table: Table<'lua>,
    idx: u64,
}

impl<'lua> serde::ser::SerializeTupleVariant for SerializeTupleVariant<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.table.set(self.idx, to_value(self.lua, value)?)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Value<'lua>> {
        let table = self.lua.create_table()?;
        table.set(self.name, self.table)?;
        Ok(Value::Table(table))
    }
}

pub struct SerializeMap<'lua> {
    lua: Context<'lua>,
    table: Table<'lua>,
    next_key: Option<Value<'lua>>,
}

impl<'lua> serde::ser::SerializeMap for SerializeMap<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.next_key = Some(to_value(self.lua, key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.table.set(key, to_value(self.lua, value)?)?;
        Ok(())
    }

    fn end(self) -> Result<Value<'lua>> {
        Ok(Value::Table(self.table))
    }
}

impl<'lua> serde::ser::SerializeStruct for SerializeMap<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value<'lua>> {
        serde::ser::SerializeMap::end(self)
    }
}

pub struct SerializeStructVariant<'lua> {
    lua: Context<'lua>,
    name: LuaString<'lua>,
    table: Table<'lua>,
}

impl<'lua> serde::ser::SerializeStructVariant for SerializeStructVariant<'lua> {
    type Ok = Value<'lua>;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.table.set(key, to_value(self.lua, value)?)?;
        Ok(())
    }

    fn end(self) -> Result<Value<'lua>> {
        let table = self.lua.create_table()?;
        table.set(self.name, self.table)?;
        Ok(Value::Table(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlua::Lua;
    use ToLuaValue;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };

        let lua = Lua::new();
        lua.context(|lua| {
            let value = to_value(lua, &test).unwrap();
            lua.globals().set("value", value).unwrap();
            lua.load(
                r#"
                assert(value["int"] == 1)
                assert(value["seq"][1] == "a")
                assert(value["seq"][2] == "b")
            "#,
            )
            .exec()
        })
        .unwrap()
    }

    #[test]
    fn test_num() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let lua = Lua::new();

        lua.context(|lua| {
            let u = E::Unit;
            let value = to_value(lua, &u).unwrap();
            lua.globals().set("value", value).unwrap();
            lua.load(
                r#"
                assert(value == "Unit")
            "#,
            )
            .exec()
            .unwrap();

            let n = E::Newtype(1);
            let value = to_value(lua, &n).unwrap();
            lua.globals().set("value", value).unwrap();
            lua.load(
                r#"
                assert(value["Newtype"] == 1)
            "#,
            )
            .exec()
            .unwrap();

            let t = E::Tuple(1, 2);
            let value = lua.to_value(t).unwrap();
            lua.globals().set("value", value).unwrap();
            lua.load(
                r#"
                assert(value["Tuple"][1] == 1)
                assert(value["Tuple"][2] == 2)
            "#,
            )
            .exec()
            .unwrap();

            let s = E::Struct { a: 1 };
            let value = to_value(lua, &s).unwrap();
            lua.globals().set("value", value).unwrap();
            lua.load(
                r#"
                assert(value["Struct"]["a"] == 1)
            "#,
            )
            .exec()
        })
        .unwrap();
    }
}
