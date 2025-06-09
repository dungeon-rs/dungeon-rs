#![cfg_attr(doc, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod deserialize;
mod error;
mod format;
mod serialize;

pub use deserialize::*;
pub use error::*;
pub use format::*;
pub use serde::{Deserialize, Serialize};
pub use serialize::*;
