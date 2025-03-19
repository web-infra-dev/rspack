#[cfg(not(feature = "noop"))]
pub use rspack_macros::{cacheable, cacheable_dyn};
#[cfg(feature = "noop")]
pub use rspack_macros::{disable_cacheable as cacheable, disable_cacheable_dyn as cacheable_dyn};
pub mod r#dyn;
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
pub use xxhash_rust;
