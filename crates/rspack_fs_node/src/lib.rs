#![allow(clippy::unwrap_in_result)]

mod node;

use std::{marker::PhantomData, path::Path};

use napi::Env;
use node::{NodeFS, NodeFSRef, TryIntoNodeFSRef};
use rspack_fs::{sync::WritableFileSystem, Error, Result};

pub struct NodeWritableFileSystem {
  env: Env,
  fs_ref: NodeFSRef,
  _data: PhantomData<*mut ()>,
}

impl NodeWritableFileSystem {
  pub fn new(env: Env, fs: NodeFS) -> napi::Result<Self> {
    Ok(Self {
      env,
      fs_ref: fs.try_into_node_fs_ref(&env)?,
      _data: PhantomData,
    })
  }
}

impl WritableFileSystem for NodeWritableFileSystem {
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    let dir = dir.as_ref().to_string_lossy();
    let mkdir = self.fs_ref.mkdir.get().expect("Failed to get mkdir");
    mkdir
      .call(
        None,
        &[self
          .env
          .create_string(&dir)
          .expect("Failed to create string")],
      )
      .map_err(|err| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          err.to_string(),
        ))
      })?;

    Ok(())
  }

  fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    let dir = dir.as_ref().to_string_lossy();
    let mkdirp = self.fs_ref.mkdirp.get().expect("Failed to get mkdirp");
    mkdirp
      .call(
        None,
        &[self
          .env
          .create_string(&dir)
          .expect("Failed to create string")],
      )
      .map_err(|err| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          err.to_string(),
        ))
      })?;

    Ok(())
  }

  fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()> {
    let file = file.as_ref().to_string_lossy();
    let buf = data.as_ref().to_vec();
    let write_file = self
      .fs_ref
      .write_file
      .get()
      .expect("Failed to get write_file");

    write_file
      .call(
        None,
        &[
          self
            .env
            .create_string(&file)
            .expect("Failed to create string")
            .into_unknown(),
          self
            .env
            .create_buffer_with_data(buf)
            .expect("Failed to create buffer")
            .into_unknown(),
        ],
      )
      .map_err(|err| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          err.to_string(),
        ))
      })?;

    Ok(())
  }
}

#[cfg(node)]
mod node_test {
  use napi::{bindgen_prelude::Buffer, Env};
  use napi_derive::napi;

  use super::*;

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
}

#[cfg(node)]
#[doc(hidden)]
pub use node_test::*;
