/// Available serialisation formats.
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum SerializationFormat {
    /// (de)serialize to/from JSON.
    #[default]
    JSON,
    /// (de)serialize to/from [MessagePack](https://msgpack.org/).
    MessagePack,
    /// (de)serialize to/from [TOML](https://toml.io/en/).
    Toml,
}
