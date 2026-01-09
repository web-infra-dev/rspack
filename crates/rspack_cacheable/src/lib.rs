#[cfg(feature = "noop")]
pub use rspack_cacheable_macros::{
  disable_cacheable as cacheable, disable_cacheable_dyn as cacheable_dyn,
};
pub use rspack_cacheable_macros::{
  disable_cacheable, disable_cacheable_dyn, enable_cacheable, enable_cacheable_dyn,
};
#[cfg(not(feature = "noop"))]
pub use rspack_cacheable_macros::{
  enable_cacheable as cacheable, enable_cacheable_dyn as cacheable_dyn,
};
pub mod r#dyn;
pub mod utils;
pub mod with;

mod context;
mod deserialize;
mod error;
mod serialize;

#[doc(hidden)]
pub mod __private {
  #[doc(hidden)]
  pub extern crate inventory;
  #[doc(hidden)]
  pub extern crate rkyv;
}

#[cfg(not(feature = "noop"))]
pub use deserialize::from_bytes;
#[cfg(feature = "noop")]
pub fn from_bytes<T, C: CacheableContext>(_bytes: &[u8], _context: &C) -> Result<T> {
  let _ = deserialize::from_bytes::<u8, u8>;
  panic!("Cannot use from_bytes when noop feature is enabled")
}

#[cfg(not(feature = "noop"))]
pub use serialize::to_bytes;
#[cfg(feature = "noop")]
pub fn to_bytes<T, C: CacheableContext>(_value: &T, _ctx: &C) -> Result<Vec<u8>> {
  let _ = serialize::to_bytes::<u8, u8>;
  panic!("Cannot use to_bytes when noop feature is enabled")
}

pub use context::{CacheableContext, ContextGuard};
pub use deserialize::{Deserializer, Validator};
pub use error::{Error, Result};
pub use serialize::Serializer;
pub use xxhash_rust;
