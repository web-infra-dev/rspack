use std::{
  ops::{Deref, DerefMut},
  path::PathBuf,
};

use crate::contextify;

#[derive(Debug, Default)]
pub struct Context {
  inner: PathBuf,
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
