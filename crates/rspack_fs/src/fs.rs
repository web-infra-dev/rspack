use std::fmt::Debug;

use crate::{ReadableFileSystem, WritableFileSystem};

pub trait FileSystem: ReadableFileSystem + WritableFileSystem + Debug + Sync + Send {}
