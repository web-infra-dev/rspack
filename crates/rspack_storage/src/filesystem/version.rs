use std::fmt;

/// Filesystem persistent-cache directory in the form
/// `rspack_v_<compiler scope hash>_<version hash>`.
///
/// The scope hash identifies the compiler that owns the directory, while the
/// version hash identifies a cache-compatible configuration of that compiler.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(String);

impl Version {
  const PREFIX: &str = "rspack_v_";
  const HASH_LEN: usize = 16;
  const SCOPE_SEPARATOR: char = '_';

  pub fn new(scope: impl AsRef<str>, hash: impl AsRef<str>) -> Self {
    let scope = scope.as_ref();
    let hash = hash.as_ref();
    assert!(
      Self::is_valid_hash(scope),
      "invalid persistent cache version scope"
    );
    assert!(
      Self::is_valid_hash(hash),
      "invalid persistent cache version hash"
    );
    Self(format!(
      "{}{scope}{}{hash}",
      Self::PREFIX,
      Self::SCOPE_SEPARATOR
    ))
  }

  pub fn parse(value: impl AsRef<str>) -> Option<Self> {
    let value = value.as_ref();
    Self::is_valid(value).then(|| Self(value.to_string()))
  }

  pub fn is_valid(value: impl AsRef<str>) -> bool {
    let Some(value) = value.as_ref().strip_prefix(Self::PREFIX) else {
      return false;
    };

    let Some((scope, hash)) = value.split_once(Self::SCOPE_SEPARATOR) else {
      return false;
    };

    Self::is_valid_hash(scope) && Self::is_valid_hash(hash)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn scope(&self) -> &str {
    self
      .0
      .strip_prefix(Self::PREFIX)
      .and_then(|value| value.split_once(Self::SCOPE_SEPARATOR))
      .map(|(scope, _)| scope)
      .expect("validated persistent cache version should have a scope")
  }

  pub fn has_same_scope(&self, other: &Self) -> bool {
    self.scope() == other.scope()
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

#[cfg(test)]
mod tests {
  use super::Version;

  #[test]
  fn parses_scoped_versions() {
    let version = Version::new("aaaaaaaaaaaaaaaa", "0000000000000001");

    assert_eq!(
      version.as_str(),
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001"
    );
    assert_eq!(version.scope(), "aaaaaaaaaaaaaaaa");
    assert_eq!(Version::parse(version.as_str()), Some(version));
  }

  #[test]
  fn compares_version_scopes() {
    let a_v1 = Version::new("aaaaaaaaaaaaaaaa", "0000000000000001");
    let a_v2 = Version::new("aaaaaaaaaaaaaaaa", "0000000000000002");
    let b_v1 = Version::new("bbbbbbbbbbbbbbbb", "0000000000000001");

    assert!(a_v1.has_same_scope(&a_v2));
    assert!(!a_v1.has_same_scope(&b_v1));
  }

  #[test]
  fn rejects_invalid_scoped_versions() {
    assert!(!Version::is_valid("rspack_v_0000000000000001"));
    assert!(!Version::is_valid(
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001_extra"
    ));
    assert!(!Version::is_valid("rspack_v_invalid_0000000000000001"));
  }
}
