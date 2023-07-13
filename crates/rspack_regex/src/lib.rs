#![feature(let_chains)]

use std::hash::Hash;

use rspack_error::Error;
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

mod algo;

/// Using wrapper type required by [TryFrom] trait
#[derive(Debug, Clone)]
pub struct RspackRegex {
  pub expr: String,
  pub flags: String,
  pub algo: Algo,
}

impl RspackRegex {
  pub fn test(&self, text: &str) -> bool {
    self.algo.test(text)
  }

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    Ok(Self {
      expr: expr.to_string(),
      flags: flags.to_string(),
      algo: Algo::new(expr, flags)?,
    })
  }

  pub fn new(expr: &str) -> Result<Self, Error> {
    Self::with_flags(expr, "")
  }
}

impl TryFrom<&SwcRegex> for RspackRegex {
  type Error = Error;

  fn try_from(value: &SwcRegex) -> Result<Self, Self::Error> {
    RspackRegex::with_flags(value.exp.as_ref(), value.flags.as_ref())
  }
}

impl TryFrom<SwcRegex> for RspackRegex {
  type Error = Error;

  fn try_from(value: SwcRegex) -> Result<Self, Self::Error> {
    RspackRegex::with_flags(value.exp.as_ref(), value.flags.as_ref())
  }
}

impl Hash for RspackRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.expr.hash(state);
    self.flags.hash(state);
  }
}
