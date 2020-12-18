use serde;
use serde::de::IntoDeserializer;

use rlua::{TablePairs, TableSequence, Value};

use error::{Error, Result};

pub struct Deserializer<'lua> {
    pub value: Value<'lua>,
}

impl<'lua, 'de> serde::Deserializer<'de> for Deserializer<'lua> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Nil => visitor.visit_unit(),
            Value::Boolean(v) => visitor.visit_bool(v),
            Value::Integer(v) => visitor.visit_i64(v),
            Value::Number(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_str(v.to_str()?),
            Value::Table(v) => {
                let len = v.len()? as usize;
                let mut deserializer = MapDeserializer(v.pairs(), None);
                let map = visitor.visit_map(&mut deserializer)?;
                let remaining = deserializer.0.count();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in array",
                    ))
                }
            }
            _ => Err(serde::de::Error::custom("invalid value type")),
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let (variant, value) = match self.value {
            Value::Table(value) => {
                let mut iter = value.pairs::<String, Value>();
                let (variant, value) = match iter.next() {
                    Some(v) => v?,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Map,
                            &"map with a single key",
                        ))
                    }
                };

                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant.to_str()?.to_owned(), None),
            _ => return Err(serde::de::Error::custom("bad enum value")),
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Table(v) => {
                let len = v.len()? as usize;
                let mut deserializer = SeqDeserializer(v.sequence_values());
                let seq = visitor.visit_seq(&mut deserializer)?;
                let remaining = deserializer.0.count();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in array",
                    ))
                }
            }
            _ => Err(serde::de::Error::custom("invalid value type")),
        }
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct
        map struct identifier ignored_any
    }
}

struct SeqDeserializer<'lua>(TableSequence<'lua, Value<'lua>>);

impl<'lua, 'de> serde::de::SeqAccess<'de> for SeqDeserializer<'lua> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(value) => seed.deserialize(Deserializer { value: value? }).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.0.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer<'lua>(
    TablePairs<'lua, Value<'lua>, Value<'lua>>,
    Option<Value<'lua>>,
);

impl<'lua, 'de> serde::de::MapAccess<'de> for MapDeserializer<'lua> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(item) => {
                let (key, value) = item?;
                self.1 = Some(value);
                let key_de = Deserializer { value: key };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.1.take() {
            Some(value) => seed.deserialize(Deserializer { value }),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.0.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct EnumDeserializer<'lua> {
    variant: String,
    value: Option<Value<'lua>>,
}

impl<'lua, 'de> serde::de::EnumAccess<'de> for EnumDeserializer<'lua> {
    type Error = Error;
    type Variant = VariantDeserializer<'lua>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant)>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let variant_access = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, variant_access))
    }
}

struct VariantDeserializer<'lua> {
    value: Option<Value<'lua>>,
}

impl<'lua, 'de> serde::de::VariantAccess<'de> for VariantDeserializer<'lua> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            Some(_) => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::NewtypeVariant,
                &"unit variant",
            )),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(Deserializer { value }),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(value) => serde::Deserializer::deserialize_seq(Deserializer { value }, visitor),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(value) => serde::Deserializer::deserialize_map(Deserializer { value }, visitor),
            None => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use rlua::Lua;

    use from_value;
    use FromLuaValue;

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
            map: std::collections::HashMap<i32, i32>,
            empty: Vec<()>,
        }

        let expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
            map: vec![(1, 2), (4, 1)].into_iter().collect(),
            empty: vec![],
        };

        println!("{:?}", expected);
        let lua = Lua::new();
        lua.context(|lua| {
            let value = lua
                .load(
                    r#"
                a = {}
                a.int = 1
                a.seq = {"a", "b"}
                a.map = {2, [4]=1}
                a.empty = {}
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);
        });
    }

    #[test]
    fn test_tuple() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Rgb(u8, u8, u8);

        let lua = Lua::new();
        lua.context(|lua| {
            let expected = Rgb(1, 2, 3);
            let value = lua
                .load(
                    r#"
                a = {1, 2, 3}
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);

            let expected = (1, 2, 3);
            let value = lua
                .load(
                    r#"
                a = {1, 2, 3}
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);
        });
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let lua = Lua::new();
        lua.context(|lua| {
            let expected = E::Unit;
            let value = lua
                .load(
                    r#"
                return "Unit"
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);

            let expected = E::Newtype(1);
            let value = lua
                .load(
                    r#"
                a = {}
                a["Newtype"] = 1
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);

            let expected = E::Tuple(1, 2);
            let value = lua
                .load(
                    r#"
                a = {}
                a["Tuple"] = {1, 2}
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = from_value(value).unwrap();
            assert_eq!(expected, got);

            let expected = E::Struct { a: 1 };
            let value: rlua::Value = lua
                .load(
                    r#"
                a = {}
                a["Struct"] = {}
                a["Struct"]["a"] = 1
                return a
            "#,
                )
                .eval()
                .unwrap();
            let got = value.from_value().unwrap();
            assert_eq!(expected, got);
        });
    }
}
