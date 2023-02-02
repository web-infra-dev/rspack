use std::{
  ops::{Deref, DerefMut},
  path::{Path, PathBuf},
};

use crate::contextify;

#[derive(Debug, Default)]
pub struct Context {
  inner: PathBuf,
}

impl Context {
  pub fn new(inner: PathBuf) -> Self {
    Self { inner }
  }
}

impl AsRef<Path> for Context {
  fn as_ref(&self) -> &Path {
    &self.inner
  }
}

impl From<String> for Context {
  fn from(v: String) -> Self {
    Self {
      inner: PathBuf::from(v),
    }
  }
}

impl Deref for Context {
  type Target = PathBuf;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for Context {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl Context {
  pub fn shorten(&self, request: &str) -> String {
    contextify(&self.inner, request)
  }
}
