use std::{marker::PhantomData, path::Path};

use napi::Env;
use rspack_fs::{sync::WritableFileSystem, Error, Result};

use crate::node::{NodeFS, NodeFSRef, TryIntoNodeFSRef};

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
  fn create_dir(&self, dir: &Path) -> Result<()> {
    let dir = dir.to_string_lossy();
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

  fn create_dir_all(&self, dir: &Path) -> Result<()> {
    let dir = dir.to_string_lossy();
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

  fn write(&self, file: &Path, data: &[u8]) -> Result<()> {
    let file = file.to_string_lossy();
    let buf = data.to_vec();
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
