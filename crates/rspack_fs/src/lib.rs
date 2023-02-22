mod macros;

cfg_async! {
  pub mod r#async;
  pub use r#async::{AsyncFileSystem, AsyncReadableFileSystem, AsyncWritableFileSystem};
}
pub mod sync;
pub use sync::{FileSystem, ReadableFileSystem, WritableFileSystem};

mod error;
pub use error::{Error, Result};

cfg_native! {
  mod native;
  pub use native::{NativeFileSystem};

  #[cfg(feature = "async")]
  pub use native::AsyncNativeFileSystem;
}
