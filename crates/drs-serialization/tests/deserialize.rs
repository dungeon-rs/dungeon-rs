#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use drs_serialization::{SerializationFormat, deserialize};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Foo {
    pub bar: String,
    pub baz: u32,
}

#[test]
fn deserialize_json() {
    let json = br#"{"bar": "baz", "baz": 123}"#;

    let deserialized = deserialize::<Foo>(json, &SerializationFormat::JSON).unwrap();
    assert_eq!(deserialized.bar, "baz");
    assert_eq!(deserialized.baz, 123);
}

#[test]
fn deserialize_messagepack() {
    let msgpack: &[u8] = &[146, 163, 98, 97, 114, 10];
    let deserialized = deserialize::<Foo>(msgpack, &SerializationFormat::MessagePack).unwrap();
    assert_eq!(deserialized.bar, "bar");
    assert_eq!(deserialized.baz, 10);
}

#[test]
fn deserialize_toml() {
    let toml = br#"bar = "baz"
baz = 10"#;
    let deserialized = deserialize::<Foo>(toml, &SerializationFormat::Toml).unwrap();
    assert_eq!(deserialized.bar, "baz");
    assert_eq!(deserialized.baz, 10);
}
