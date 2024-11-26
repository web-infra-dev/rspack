use std::fmt::Debug;

use crate::{ReadableFileSystem, WritableFileSystem, WritableFileSystemExt};

pub trait FileSystem: ReadableFileSystem + WritableFileSystem + Debug + Sync + Send {}
pub trait FileSystemExt: FileSystem + WritableFileSystemExt {}
impl<T: FileSystem + WritableFileSystemExt> FileSystemExt for T {}
