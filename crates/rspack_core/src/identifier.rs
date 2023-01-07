use std::{convert::From, fmt, ops::Deref};

use ustr::Ustr;

pub trait Identifiable {
  fn identifier(&self) -> Identifier;
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Identifier(Ustr);

impl Deref for Identifier {
  type Target = Ustr;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Ustr> for Identifier {
  fn from(s: Ustr) -> Self {
    Self(s)
  }
}

impl From<&str> for Identifier {
  fn from(s: &str) -> Self {
    Self(Ustr::from(s))
  }
}

impl From<String> for Identifier {
  fn from(s: String) -> Self {
    Self(Ustr::from(&s))
  }
}

impl From<Identifier> for Ustr {
  fn from(val: Identifier) -> Self {
    val.0
  }
}

impl fmt::Display for Identifier {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0.as_str())
  }
}
