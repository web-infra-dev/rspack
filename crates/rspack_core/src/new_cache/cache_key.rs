use std::{fmt, ops::Deref, sync::Arc};

/// Immutable, reference-counted cache identifier.
///
/// Cloning a key only increments its reference count. The string is released
/// when its last key is dropped.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CacheKey(Arc<str>);

impl CacheKey {
  pub fn new(value: &str) -> Self {
    Self(Arc::from(value))
  }

  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl fmt::Debug for CacheKey {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_tuple("CacheKey")
      .field(&self.as_str())
      .finish()
  }
}

impl fmt::Display for CacheKey {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.as_str())
  }
}

impl AsRef<str> for CacheKey {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl Deref for CacheKey {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl From<&str> for CacheKey {
  fn from(value: &str) -> Self {
    Self::new(value)
  }
}

impl From<String> for CacheKey {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}

impl From<Arc<str>> for CacheKey {
  fn from(value: Arc<str>) -> Self {
    Self(value)
  }
}
