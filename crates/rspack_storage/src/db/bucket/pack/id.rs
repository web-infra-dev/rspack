use super::{Error, Result};

/// Unique identifier for a pack file.
///
/// Pack IDs are sequential integers starting from 1.
/// ID 0 is reserved for the "hot pack" which stores frequently modified data.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd)]
pub struct PackId(usize);

impl std::fmt::Display for PackId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::str::FromStr for PackId {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self> {
    let inner = s
      .parse()
      .map_err(|_| Error::InvalidFormat(format!("Failed to parse PackId from '{}'", s)))?;
    Ok(Self(inner))
  }
}

impl std::ops::Add<usize> for PackId {
  type Output = Self;
  fn add(self, rhs: usize) -> Self::Output {
    Self(self.0 + rhs)
  }
}

impl PackId {
  pub const fn new(inner: usize) -> Self {
    Self(inner)
  }

  /// Returns the filename for this pack (e.g., "1.pack")
  pub fn pack_name(&self) -> String {
    format!("{}.pack", self.0)
  }
}

#[cfg(test)]
mod test {
  use super::PackId;
  #[test]
  fn test_pack_id() {
    let id = PackId::new(10);
    let other_id = id.to_string().parse().expect("should deserialize success");
    assert_eq!(id, other_id);
  }
}
