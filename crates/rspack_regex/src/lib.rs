#![feature(let_chains)]

use rspack_error::Error;
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

mod algo;

/// Using wrapper type required by [TryFrom] trait
#[derive(Debug, Clone, Hash)]
pub struct RspackRegex {
  pub algo: Algo,
  raw: String,
}

impl RspackRegex {
  pub fn test(&self, text: &str) -> bool {
    self.algo.test(text)
  }

  pub fn global(&self) -> bool {
    self.algo.global()
  }

  pub fn sticky(&self) -> bool {
    self.algo.sticky()
  }

  pub fn raw(&self) -> &str {
    &self.raw
  }

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    let mut chars = flags.chars().collect::<Vec<char>>();
    chars.sort();
    let raw = format!("{expr}|{}", chars.into_iter().collect::<String>());
    Ok(Self {
      raw,
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
