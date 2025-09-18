use std::{
  fmt,
  ops::Deref,
  path::{Path, PathBuf},
};

use rspack_cacheable::{cacheable, with::AsPreset};
use rspack_loader_runner::ResourceData;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use rspack_util::atom::Atom;

use crate::{contextify, parse_resource};

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Context {
  #[cacheable(with=AsPreset)]
  inner: Atom,
}

impl Context {
  pub fn new(inner: Atom) -> Self {
    Self { inner }
  }

  pub fn as_str(&self) -> &str {
    self.as_ref()
  }

  pub fn as_path(&self) -> &Utf8Path {
    Utf8Path::new(self.as_str())
  }
}

impl AsRef<Utf8Path> for Context {
  fn as_ref(&self) -> &Utf8Path {
    Utf8Path::new(self.inner.as_str())
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

impl From<Utf8PathBuf> for Context {
  fn from(v: Utf8PathBuf) -> Self {
    Self {
      inner: v.as_str().into(),
    }
  }
}

impl From<&Utf8Path> for Context {
  fn from(v: &Utf8Path) -> Self {
    Self {
      inner: v.as_str().into(),
    }
  }
}

impl From<PathBuf> for Context {
  fn from(value: PathBuf) -> Self {
    value.assert_utf8().into()
  }
}

impl From<&Path> for Context {
  fn from(value: &Path) -> Self {
    value.assert_utf8().into()
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
  let Some(i) = path.rfind(['/', '\\']) else {
    return path;
  };
  let c = match path.as_bytes().get(i) {
    Some(b'/') => '/',
    Some(b'\\') => '\\',
    _ => unreachable!("path delimiter should be slash or backslash"),
  };
  let i2 = path.find(c).expect("should exist");
  if i == i2 {
    return &path[..i + 1];
  }
  &path[..i]
}

pub fn get_context(resource_data: &ResourceData) -> Context {
  if let Some(resource_path) = resource_data.path() {
    dirname(resource_path.as_str()).into()
  } else if let Some(parsed) = parse_resource(resource_data.resource()) {
    dirname(parsed.path.as_str()).into()
  } else {
    Context::from("")
  }
}

#[test]
fn dirname_data_uri() {
  let d = dirname("data:text/javascript,import \"a\"");
  assert_eq!(d, "data:text/");
}

#[test]
fn dirname_non_ascii_path() {
  let d = dirname("C:/非常长的中文来测试宽字符溢出问题/src/index.js");
  assert_eq!(d, "C:/非常长的中文来测试宽字符溢出问题/src");
}
