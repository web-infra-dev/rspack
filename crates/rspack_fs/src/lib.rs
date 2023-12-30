mod macros;

cfg_async! {
  pub mod r#async;
  pub use r#async::{AsyncFileSystem, AsyncReadableFileSystem, AsyncWritableFileSystem};
}
pub mod sync;
pub use sync::{FileSystem, ReadableFileSystem, WritableFileSystem};

mod error;
mod metadata;

pub use error::{Error, Result};
pub use metadata::FSMetadata;

cfg_native! {
  mod native;
  pub use native::{NativeFileSystem};

  #[cfg(feature = "async")]
  pub use native::AsyncNativeFileSystem;
}
