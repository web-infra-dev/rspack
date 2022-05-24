use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum BundleMode {
  Dev,
  Prod,
  None,
}

impl BundleMode {
  pub fn is_dev(&self) -> bool {
    matches!(self, Self::Dev)
  }
  pub fn is_prod(&self) -> bool {
    matches!(self, Self::Prod)
  }
  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }
}

impl TryFrom<&str> for BundleMode {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "production" => Ok(Self::Prod),
      "development" => Ok(Self::Dev),
      "none" => Ok(Self::None),
      _ => Err(format!("unexpected value: {:?}", value)),
    }
  }
}

impl FromStr for BundleMode {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.try_into()
  }
}
