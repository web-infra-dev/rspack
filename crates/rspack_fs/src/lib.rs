mod fs;
pub use fs::{FileSystem, ReadableFileSystem, WritableFileSystem};
mod r#async;
pub use r#async::{AsyncFileSystem, AsyncReadableFileSystem, AsyncWritableFileSystem};

mod sync;
pub use sync::{SyncFileSystem, SyncReadableFileSystem, SyncWritableFileSystem};

mod file_metadata;
pub use file_metadata::FileMetadata;

mod macros;

mod native_fs;
pub use native_fs::NativeFileSystem;

mod memory_fs;
pub use memory_fs::MemoryFileSystem;

mod error;
pub use error::{Error, Result};
