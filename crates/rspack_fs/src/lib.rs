mod r#async;
pub use r#async::{AsyncFileSystem, AsyncReadableFileSystem, AsyncWritableFileSystem};

mod sync;
pub use sync::{FileSystem, ReadableFileSystem, WritableFileSystem};

mod file_metadata;
pub use file_metadata::FileMetadata;

mod macros;

mod native_fs;
pub use native_fs::NativeFileSystem;

mod error;
pub use error::{Error, Result};
