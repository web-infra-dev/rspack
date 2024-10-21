pub use rspack_macros::{cacheable, cacheable_dyn};
pub mod r#dyn;
pub mod utils;
pub mod with;

mod context;
mod deserialize;
mod serialize;

#[doc(hidden)]
pub mod __private {
  #[doc(hidden)]
  pub extern crate inventory;
  #[doc(hidden)]
  pub extern crate rkyv;
}

pub use deserialize::{from_bytes, DeserializeError, Deserializer, Validator};
pub use serialize::{to_bytes, SerializeError, Serializer};
