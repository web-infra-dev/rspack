use std::fmt;

use rspack_fs::RelPath;

/// Filesystem persistent-cache directory name derived from a compiler path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheDirectory(RelPath);

impl CacheDirectory {
  const PREFIX: &str = "rspack_v_";
  const HASH_LEN: usize = 16;

  pub fn new(hash: impl AsRef<str>) -> Self {
    let hash = hash.as_ref();
    assert!(
      Self::is_valid_hash(hash),
      "invalid persistent cache directory hash"
    );
    Self(
      RelPath::new_contained(&format!("{}{hash}", Self::PREFIX))
        .expect("valid persistent cache directory"),
    )
  }

  pub fn parse(value: impl AsRef<str>) -> Option<Self> {
    let value = value.as_ref();
    Self::is_valid(value).then(|| {
      Self(RelPath::new_contained(value).expect("validated cache directory is contained RelPath"))
    })
  }

  pub fn is_valid(value: impl AsRef<str>) -> bool {
    let Some(value) = value.as_ref().strip_prefix(Self::PREFIX) else {
      return false;
    };

    Self::is_valid_hash(value)
  }

  pub fn as_rel_path(&self) -> &RelPath {
    &self.0
  }

  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }

  fn is_valid_hash(hash: &str) -> bool {
    hash.len() == Self::HASH_LEN
      && hash
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
  }
}

impl fmt::Display for CacheDirectory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl AsRef<str> for CacheDirectory {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<RelPath> for CacheDirectory {
  fn as_ref(&self) -> &RelPath {
    self.as_rel_path()
  }
}
