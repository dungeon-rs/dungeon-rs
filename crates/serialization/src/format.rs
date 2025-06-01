#[non_exhaustive]
#[derive(Debug, Default)]
pub enum SerializationFormat {
    #[default]
    JSON,
    MessagePack,
}
