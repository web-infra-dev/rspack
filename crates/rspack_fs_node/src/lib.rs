#![allow(clippy::unwrap_in_result)]

use rspack_fs::cfg_async;

cfg_async! {
  mod r#async;
  pub use r#async::AsyncNodeWritableFileSystem;
}
mod sync;
pub use sync::NodeWritableFileSystem;

mod node;
pub use node::NodeFS;

cfg_async! {
  pub use node::ThreadsafeNodeFS;
}

#[cfg(node)]
mod node_test {
  use napi::{bindgen_prelude::Buffer, Env};
  use napi_derive::napi;
  use rspack_fs::{r#async::AsyncWritableFileSystem, sync::WritableFileSystem};

  use crate::{
    node::NodeFS, AsyncNodeWritableFileSystem, NodeWritableFileSystem, ThreadsafeNodeFS,
  };

  #[napi]
  pub struct TestFS {
    writable_fs: NodeWritableFileSystem,
  }

  #[napi]
  impl TestFS {
    #[napi(constructor)]
    pub fn new(env: Env, fs: NodeFS) -> Self {
      Self {
        writable_fs: NodeWritableFileSystem::new(env, fs).unwrap(),
      }
    }

    #[napi]
    pub fn write_sync(&self, file: String, data: Buffer) {
      self.writable_fs.write(file, data).unwrap();
    }

    #[napi]
    pub fn mkdir_sync(&self, file: String) {
      self.writable_fs.create_dir(file).unwrap();
    }

    #[napi]
    pub fn mkdirp_sync(&self, file: String) {
      self.writable_fs.create_dir_all(file).unwrap();
    }
  }

  #[napi]
  pub struct TestThreadsafeFS {
    writable_fs: AsyncNodeWritableFileSystem,
  }

  #[napi]
  impl TestThreadsafeFS {
    #[napi(constructor)]
    pub fn new(env: Env, fs: ThreadsafeNodeFS) -> Self {
      Self {
        writable_fs: AsyncNodeWritableFileSystem::new(fs).unwrap(),
      }
    }

    #[napi]
    pub async fn write(&self, file: String, data: Buffer) {
      self.writable_fs.write(file, data).await.unwrap();
    }

    #[napi]
    pub async fn mkdir(&self, file: String) {
      self.writable_fs.create_dir(file).await.unwrap();
    }

    #[napi]
    pub async fn mkdirp(&self, file: String) {
      self.writable_fs.create_dir_all(file).await.unwrap();
    }

    #[napi]
    pub async fn remove_file(&self, file: String) {
      self.writable_fs.remove_file(file).await.unwrap();
    }

    #[napi]
    pub async fn remove_dir_all(&self, file: String) {
      self.writable_fs.remove_dir_all(file).await.unwrap();
    }
  }
}

#[cfg(node)]
#[doc(hidden)]
pub use node_test::*;
