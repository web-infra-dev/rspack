#[macro_use]
#[doc(hidden)]
pub mod macros;

cfg_async! {
  pub mod r#async;
}
pub mod sync;

mod error;
pub use error::{Error, Result};

cfg_native! {
  mod native;
  pub use native::NativeFileSystem;
}
