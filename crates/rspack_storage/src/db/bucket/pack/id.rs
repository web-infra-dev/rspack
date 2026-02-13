use super::{Error, Result};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PackId(usize);

impl std::fmt::Display for PackId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl TryFrom<&str> for PackId {
  type Error = Error;
  fn try_from(value: &str) -> Result<Self> {
    let Ok(inner) = value.parse::<usize>() else {
      return Err(Error::InvalidFormat(format!(
        "parse pack id failed, source is {}",
        value
      )));
    };
    Ok(Self(inner))
  }
}

impl PackId {
  pub const fn new(inner: usize) -> Self {
    Self(inner)
  }

  pub fn pack_name(&self) -> String {
    format!("{}.pack", self.0)
  }

  pub fn inner(&self) -> usize {
    self.0
  }
}
