use std::fmt;

/// Filesystem persistent-cache directory name derived from a compiler path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheDirectory(String);

impl CacheDirectory {
  const PREFIX: &str = "rspack_v_";
  const HASH_LEN: usize = 16;

  pub fn new(hash: impl AsRef<str>) -> Self {
    let hash = hash.as_ref();
    assert!(
      Self::is_valid_hash(hash),
      "invalid persistent cache directory hash"
    );
    Self(format!("{}{hash}", Self::PREFIX))
  }

  pub fn parse(value: impl AsRef<str>) -> Option<Self> {
    let value = value.as_ref();
    Self::is_valid(value).then(|| Self(value.to_string()))
  }

  pub fn is_valid(value: impl AsRef<str>) -> bool {
    let Some(value) = value.as_ref().strip_prefix(Self::PREFIX) else {
      return false;
    };

    Self::is_valid_hash(value)
  }

  pub(crate) fn is_legacy_version(value: impl AsRef<str>) -> bool {
    let Some(value) = value.as_ref().strip_prefix(Self::PREFIX) else {
      return false;
    };
    let Some((compiler_hash, version_hash)) = value.split_once('_') else {
      return false;
    };

    Self::is_valid_hash(compiler_hash) && Self::is_valid_hash(version_hash)
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

impl fmt::Display for CacheDirectory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for CacheDirectory {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

#[cfg(test)]
mod tests {
  use super::CacheDirectory;

  #[test]
  fn parses_cache_directories() {
    let directory = CacheDirectory::new("0000000000000001");

    assert_eq!(directory.as_str(), "rspack_v_0000000000000001");
    assert_eq!(CacheDirectory::parse(directory.as_str()), Some(directory));
  }

  #[test]
  fn rejects_invalid_cache_directories() {
    assert!(!CacheDirectory::is_valid("0000000000000001"));
    assert!(!CacheDirectory::is_valid(
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001_extra"
    ));
    assert!(!CacheDirectory::is_valid("rspack_v_invalid"));
  }

  #[test]
  fn recognizes_legacy_version_directories() {
    assert!(CacheDirectory::is_legacy_version(
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001"
    ));
    assert!(!CacheDirectory::is_legacy_version(
      "rspack_v_0000000000000001"
    ));
    assert!(!CacheDirectory::is_legacy_version(
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001_extra"
    ));
  }
}
