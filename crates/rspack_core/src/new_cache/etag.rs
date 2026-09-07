use std::{fmt, ops::Deref, sync::Arc};

use rspack_cacheable::cacheable;

/// Immutable validation token associated with a cached value.
#[cacheable]
#[derive(Clone, Eq, PartialEq)]
pub struct Etag(Arc<str>);

impl Etag {
  pub fn new(value: &str) -> Self {
    Self(Arc::from(value))
  }

  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }
}

impl fmt::Debug for Etag {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_tuple("Etag").field(&self.as_str()).finish()
  }
}

impl fmt::Display for Etag {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.as_str())
  }
}

impl AsRef<str> for Etag {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl Deref for Etag {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl From<&str> for Etag {
  fn from(value: &str) -> Self {
    Self::new(value)
  }
}

impl From<String> for Etag {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}

impl From<Arc<str>> for Etag {
  fn from(value: Arc<str>) -> Self {
    Self(value)
  }
}
