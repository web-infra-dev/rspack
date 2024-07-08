#![feature(let_chains)]

use std::fmt::Debug;

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
      .field(&self.to_source_string())
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

  pub fn source(&self) -> &str {
    &self.source
  }

  pub fn flags(&self) -> &str {
    &self.flags
  }

  pub fn new(expr: &str) -> Result<Self, Error> {
    Self::with_flags(expr, "")
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/dependencies/ContextDependency.js#L30
  pub fn to_source_string(&self) -> String {
    format!("/{}/{}", self.source, self.flags)
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/ContextModule.js#L192
  pub fn to_pretty_string(&self, strip_slash: bool) -> String {
    if strip_slash {
      format!("{}{}", self.source, self.flags)
    } else {
      self.to_source_string()
    }
    .replace('!', "%21")
    .replace('|', "%7C")
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
