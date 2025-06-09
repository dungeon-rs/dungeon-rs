#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use serialization::{SerializationFormat, serialize};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Foo {
    pub bar: String,
    pub baz: u32,
}

#[test]
fn serialize_json() {
    let foo = Foo {
        bar: "bar".to_string(),
        baz: 42,
    };

    let json = serialize(&foo, &SerializationFormat::JSON).unwrap();
    assert_eq!(
        json,
        br#"{
  "bar": "bar",
  "baz": 42
}"#
    );
}

#[test]
fn serialize_messagepack() {
    let foo = Foo {
        bar: "bar".to_string(),
        baz: 42,
    };

    let msgpack = serialize(&foo, &SerializationFormat::MessagePack).unwrap();
    assert_eq!(msgpack, Vec::<u8>::from([146, 163, 98, 97, 114, 42]));
}

#[test]
fn serialize_toml() {
    let foo = Foo {
        bar: "bar".to_string(),
        baz: 42,
    };

    let toml = serialize(&foo, &SerializationFormat::Toml).unwrap();
    assert_eq!(
        toml,
        br#"bar = "bar"
baz = 42
"#
    );
}
