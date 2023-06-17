#![feature(try_blocks)]

use regress::Regex;
use rspack_error::{internal_error, Error};
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

mod algo;

/// Using wrapper type required by [TryFrom] trait
#[derive(Debug, Clone)]
pub struct RspackRegex {
  pub algo: Algo,
}

impl RspackRegex {
  pub fn test(&self, text: &str) -> bool {
    self.algo.test(text)
  }

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    Regex::with_flags(expr, flags)
      .map(|regex| RspackRegex {
        algo: Algo::Regress(regex),
      })
      .map_err(|_| internal_error!("Can't construct regex `/{expr}/{flags}`"))
  }

  pub fn new(expr: &str) -> Result<Self, Error> {
    Regex::with_flags(expr, "")
      .map(|regex| RspackRegex {
        algo: Algo::Regress(regex),
      })
      .map_err(|_| internal_error!("Can't construct regex `/{}/{}`", expr, ""))
  }

  pub fn new_with_optimized(expr: &str) -> Result<Self, Error> {
    Algo::new(expr, "").map(|algo| RspackRegex { algo })
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
