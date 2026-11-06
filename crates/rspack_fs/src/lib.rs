mod read;
pub use read::ReadableFileSystem;

mod write;
pub use write::WritableFileSystem;
mod intermediate;
pub use intermediate::{
  IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream, WriteStream,
};

mod file_metadata;
pub use file_metadata::{FileMetadata, FilePermissions, current_time, system_time_to_millis};

mod walk;
pub use walk::WalkMatcher;

mod macros;

mod native_fs;
pub use native_fs::{NativeFileSystem, NativeReadStream, NativeWriteStream};

mod memory_fs;
pub use memory_fs::{MemoryFileSystem, MemoryReadStream, MemoryWriteStream};

mod noop_fs;
pub use noop_fs::NoopFileSystem;

mod error;
pub use error::{
  Error, FsResultToIoResultExt, IoResultToFsResultExt, Result, RspackResultToFsResultExt,
};
