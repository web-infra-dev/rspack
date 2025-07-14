mod read;
pub use read::ReadableFileSystem;

mod write;
pub use write::WritableFileSystem;
mod intermediate;
pub use intermediate::{
  IntermediateFileSystem, IntermediateFileSystemExtras, ReadStream, WriteStream,
};

mod file_metadata;
pub use file_metadata::{FileMetadata, FilePermissions};

mod macros;

mod native_fs;
pub use native_fs::{NativeFileSystem, NativeReadStream, NativeWriteStream};

mod memory_fs;
pub use memory_fs::{MemoryFileSystem, MemoryReadStream, MemoryWriteStream};

mod error;
pub use error::{
  Error, FsResultToIoResultExt, IoResultToFsResultExt, Result, RspackResultToFsResultExt,
};
