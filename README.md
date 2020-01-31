# rlue_serde

Implementation of [serde](https://serde.rs/) Serializer/Deserializer for [rlua::Value](https://docs.rs/rlua/0.12/rlua/enum.Value.html)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/rlua_serde.svg)](https://crates.io/crates/rlua_serde)
[![Documentation](https://docs.rs/rlua_serde/badge.svg)][dox]

More information about this crate can be found in the [crate documentation][dox].

[dox]: https://docs.rs/rlua_serde/*/rlua_serde/

## Usage

To use `rlua_serde`, first add this to your `Cargo.toml`:

```toml
[dependencies]
rlua_serde = "0.4"
```

Next, you can use `to_value`/`from_value` functions to serialize/deserialize:

```rust
#[derive(Serialize, Deserialize)]
struct Foo {
    bar: u32,
    baz: Vec<String>,
}

fn main() {
    let lua = rlua::Lua::new();
    lua.context(|lua| {
        let foo = Foo {
            bar: 42,
            baz: vec![String::from("fizz"), String::from("buzz")],
        };

        let value = rlua_serde::to_value(lua, &foo).unwrap();
        lua.globals().set("value", value).unwrap();
        lua.load(
            r#"
                assert(value["bar"] == 42)
                assert(value["baz"][2] == "buzz")
            "#).exec().unwrap();
    });
}
```
