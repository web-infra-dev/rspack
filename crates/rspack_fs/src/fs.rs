use crate::{
  AsyncFileSystem, AsyncReadableFileSystem, AsyncWritableFileSystem, SyncFileSystem,
  SyncReadableFileSystem, SyncWritableFileSystem,
};

pub trait FileSystem: AsyncFileSystem + SyncFileSystem {}

pub trait ReadableFileSystem: AsyncReadableFileSystem + SyncReadableFileSystem {}

pub trait WritableFileSystem: AsyncWritableFileSystem + SyncWritableFileSystem {}
