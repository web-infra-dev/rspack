#![feature(let_chains)]

use std::fmt::{Debug, Display};

use rspack_error::Error;
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

mod algo;

/// Using wrapper type required by [TryFrom] trait
#[derive(Clone, Hash)]
pub struct RspackRegex {
  algo: Box<Algo>,
  source: String,
  flags: String,
}

impl Debug for RspackRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("RspackRegex")
      .field(&self.to_string())
      .finish()
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
    Ok(Self {
      flags: chars.into_iter().collect::<String>(),
      source: expr.to_string(),
      algo: Box::new(Algo::new(expr, flags)?),
    })
  }

  pub fn new(expr: &str) -> Result<Self, Error> {
    Self::with_flags(expr, "")
  }

  pub fn to_pretty_string(&self, strip_slash: bool) -> String {
    if strip_slash {
      format!("{}{}", self.source, self.flags)
    } else {
      self.to_string()
    }
    .replace('!', "%21")
    .replace('|', "%7C")
  }
}

impl Display for RspackRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "/{}/{}", self.source, self.flags)
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
