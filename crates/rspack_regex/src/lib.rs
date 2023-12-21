#![feature(let_chains)]

use std::fmt::Debug;

use rspack_error::Error;
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

mod algo;

/// Using wrapper type required by [TryFrom] trait
#[derive(Clone, Hash)]
pub struct RspackRegex {
  pub algo: Algo,
  raw: String,
}

impl Debug for RspackRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("RspackRegex").field(&self.raw).finish()
  }
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

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    let mut chars = flags.chars().collect::<Vec<char>>();
    chars.sort_unstable();
    let raw = if chars.is_empty() {
      expr.to_string()
    } else {
      format!("{expr}|{}", chars.into_iter().collect::<String>())
    };
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

pub fn regexp_as_str(reg: &RspackRegex) -> &str {
  &reg.raw
}
