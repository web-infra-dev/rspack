use std::{
  borrow::Cow,
  fmt,
  ops::Deref,
  path::{Path, PathBuf},
};

use rspack_loader_runner::{get_scheme, ResourceData};
use rspack_util::atom::Atom;

use crate::{contextify, parse_resource};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Context {
  inner: Atom,
}

impl Context {
  pub fn new(inner: Atom) -> Self {
    Self { inner }
  }

  pub fn as_str(&self) -> &str {
    self.as_ref()
  }
}

impl AsRef<Path> for Context {
  fn as_ref(&self) -> &Path {
    Path::new(self.inner.as_str())
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
    Self { inner: v.into() }
  }
}

impl From<&str> for Context {
  fn from(v: &str) -> Self {
    Self { inner: v.into() }
  }
}

impl From<PathBuf> for Context {
  fn from(v: PathBuf) -> Self {
    Self {
      inner: v.to_string_lossy().into(),
    }
  }
}

impl From<&Path> for Context {
  fn from(v: &Path) -> Self {
    Self {
      inner: v.to_string_lossy().into(),
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
    contextify(self.inner.as_str(), request)
  }
}

fn dirname(path: &str) -> &str {
  if path == "/" {
    return path;
  }
  let i = path.rfind('/');
  let j = path.rfind("\\\\");
  let i2 = path.find('/');
  let j2 = path.find("\\\\");
  let (idx, is_i) = match (i, j) {
    (None, None) => return path,
    (None, Some(j)) => (j, false),
    (Some(i), None) => (i, true),
    (Some(i), Some(j)) => {
      if i > j {
        (i, true)
      } else {
        (j, false)
      }
    }
  };
  let idx2 = (if is_i { i2 } else { j2 }).expect("should have value");
  if idx == idx2 {
    return &path[..idx + 1];
  }
  &path[..idx]
}

pub fn get_context(resource_data: &ResourceData) -> Context {
  if let Some(resource_path) = &resource_data.resource_path
    && let Some(dirname) = resource_path.parent()
  {
    dirname.into()
  } else if let Some(parsed) = parse_resource(&resource_data.resource) {
    dirname(&parsed.path.to_string_lossy()).into()
  } else {
    Context::from("")
  }
}

#[test]
fn dirname_data_uri() {
  let d = dirname("data:text/javascript,import \"a\"");
  assert_eq!(d, "data:text/");
}
