# `DungeonRS serialisation`
This crate handles serialisation from and to binary representations.
It's essentially a wrapper around Serde using various formats depending on the enabled features.

These are the currently supported features:

| Format      | Doc                                  | Feature   |
|-------------|--------------------------------------|-----------|
| MessagePack | Binary format with small output size | `msgpack` |
| JSON        | Human-readable text format           | `json`    |
| TOML        | Human-readable configuration format  | `toml`    |
