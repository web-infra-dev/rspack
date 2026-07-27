use std::fmt;

/// Filesystem persistent-cache version directory name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(String);

impl Version {
  const PREFIX: &str = "rspack_v_";
  const HASH_LEN: usize = 16;
  const SCOPE_SEPARATOR: char = '_';

  pub fn new(hash: impl AsRef<str>) -> Self {
    let hash = hash.as_ref();
    assert!(
      Self::is_valid_hash(hash),
      "invalid persistent cache version hash"
    );
    Self(format!("{}{hash}", Self::PREFIX))
  }

  pub fn new_scoped(scope: impl AsRef<str>, hash: impl AsRef<str>) -> Self {
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

    if let Some((scope, hash)) = value.split_once(Self::SCOPE_SEPARATOR) {
      Self::is_valid_hash(scope) && Self::is_valid_hash(hash)
    } else {
      Self::is_valid_hash(value)
    }
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn scope(&self) -> Option<&str> {
    self
      .0
      .strip_prefix(Self::PREFIX)
      .and_then(|value| value.split_once(Self::SCOPE_SEPARATOR))
      .map(|(scope, _)| scope)
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
  fn parses_scoped_and_legacy_versions() {
    let scoped = Version::new_scoped("aaaaaaaaaaaaaaaa", "0000000000000001");
    let legacy = Version::new("0000000000000001");

    assert_eq!(
      scoped.as_str(),
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001"
    );
    assert_eq!(scoped.scope(), Some("aaaaaaaaaaaaaaaa"));
    assert_eq!(legacy.scope(), None);
    assert_eq!(Version::parse(scoped.as_str()), Some(scoped));
    assert_eq!(Version::parse(legacy.as_str()), Some(legacy));
  }

  #[test]
  fn compares_version_scopes() {
    let a_v1 = Version::new_scoped("aaaaaaaaaaaaaaaa", "0000000000000001");
    let a_v2 = Version::new_scoped("aaaaaaaaaaaaaaaa", "0000000000000002");
    let b_v1 = Version::new_scoped("bbbbbbbbbbbbbbbb", "0000000000000001");

    assert!(a_v1.has_same_scope(&a_v2));
    assert!(!a_v1.has_same_scope(&b_v1));
  }

  #[test]
  fn rejects_invalid_scoped_versions() {
    assert!(!Version::is_valid(
      "rspack_v_aaaaaaaaaaaaaaaa_0000000000000001_extra"
    ));
    assert!(!Version::is_valid("rspack_v_invalid_0000000000000001"));
  }
}
