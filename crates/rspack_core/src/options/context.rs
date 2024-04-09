use std::{
  fmt,
  ops::Deref,
  path::{Path, PathBuf},
};

use rspack_loader_runner::ResourceData;

use crate::{contextify, parse_resource};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Context {
  inner: String,
}

impl Context {
  pub fn new(inner: String) -> Self {
    Self { inner }
  }

  pub fn as_str(&self) -> &str {
    self.as_ref()
  }
}

impl AsRef<Path> for Context {
  fn as_ref(&self) -> &Path {
    Path::new(&self.inner)
  }
}

impl AsRef<str> for Context {
  fn as_ref(&self) -> &str {
    &self.inner
  }
}

impl Deref for Context {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl From<String> for Context {
  fn from(v: String) -> Self {
    Self { inner: v }
  }
}

impl From<&str> for Context {
  fn from(v: &str) -> Self {
    Self {
      inner: v.to_owned(),
    }
  }
}

impl From<Context> for String {
  fn from(v: Context) -> Self {
    v.inner
  }
}

impl From<PathBuf> for Context {
  fn from(v: PathBuf) -> Self {
    Self {
      inner: v.to_string_lossy().into_owned(),
    }
  }
}

impl From<&Path> for Context {
  fn from(v: &Path) -> Self {
    Self {
      inner: v.to_string_lossy().into_owned(),
    }
  }
}

impl fmt::Display for Context {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.inner)
  }
}

impl Context {
  pub fn shorten(&self, request: &str) -> String {
    contextify(&self.inner, request)
  }
}

pub fn get_context(resource_data: &ResourceData) -> Context {
  if let Some(dirname) = resource_data.resource_path.parent() {
    dirname.into()
  } else if let Some(parsed) = parse_resource(&resource_data.resource)
    && let Some(dirname) = parsed.path.parent()
  {
    dirname.into()
  } else {
    Context::from("")
  }
}
