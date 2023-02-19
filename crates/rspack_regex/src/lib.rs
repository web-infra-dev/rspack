use regress::{Match, Matches, Regex};
use rspack_error::{internal_error, Error};
use swc_core::ecma::ast::Regex as SwcRegex;

/// Using wrapper because I want to implement the `TryFrom` Trait
#[derive(Debug, Clone)]
pub struct RspackRegex(Regex);

impl RspackRegex {
  pub fn find(&self, str: &str) -> Option<Match> {
    self.0.find(str)
  }
  pub fn test(&self, str: &str) -> bool {
    self.find(str).is_some()
  }

  pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> Matches<'r, 't> {
    self.0.find_iter(text)
  }

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    Regex::with_flags(expr, flags)
      .map(RspackRegex)
      .map_err(|_| internal_error!("Can't construct regex `/{expr}/{flags}`"))
  }

  pub fn new(expr: &str) -> Result<Self, Error> {
    Regex::with_flags(expr, "")
      .map(RspackRegex)
      .map_err(|_| internal_error!("Can't construct regex `/{}/{}`", expr, ""))
  }
}

impl TryFrom<&SwcRegex> for RspackRegex {
  type Error = Error;

  fn try_from(value: &SwcRegex) -> Result<Self, Self::Error> {
    RspackRegex::with_flags(value.exp.as_ref(), value.flags.as_ref())
  }
}
