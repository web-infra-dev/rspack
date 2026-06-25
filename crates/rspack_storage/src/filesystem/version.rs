use std::fmt;

/// Filesystem persistent-cache version directory name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(String);

impl Version {
  const PREFIX: &str = "rspack_v_";
  const HASH_LEN: usize = 16;

  pub fn new(hash: impl AsRef<str>) -> Self {
    let hash = hash.as_ref();
    assert!(
      Self::is_valid_hash(hash),
      "invalid persistent cache version hash"
    );
    Self(format!("{}{hash}", Self::PREFIX))
  }

  pub fn parse(value: impl AsRef<str>) -> Option<Self> {
    let value = value.as_ref();
    Self::is_valid(value).then(|| Self(value.to_string()))
  }

  pub fn is_valid(value: impl AsRef<str>) -> bool {
    let Some(hash) = value.as_ref().strip_prefix(Self::PREFIX) else {
      return false;
    };

    Self::is_valid_hash(hash)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  fn is_valid_hash(hash: &str) -> bool {
    hash.len() == Self::HASH_LEN
      && hash
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
  }
}

impl fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for Version {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}
