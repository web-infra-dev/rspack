mod algo;
mod napi;

use std::fmt::Debug;

use cow_utils::CowUtils;
use rspack_cacheable::{
  cacheable,
  with::{AsString, AsStringConverter},
};
use rspack_error::Error;
use swc_experimental_ecma_ast::{Ast, Regex as SwcRegex};

use self::algo::Algo;

/// Using wrapper type required by [TryFrom] trait
#[cacheable(with=AsString)]
#[derive(Clone)]
pub struct RspackRegex {
  algo: Box<Algo>,
  pub flags: String,
  pub source: String,
}

impl PartialEq for RspackRegex {
  fn eq(&self, other: &Self) -> bool {
    self.flags == other.flags && self.source == other.source
  }
}

impl Eq for RspackRegex {}

impl std::hash::Hash for RspackRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.flags.hash(state);
    self.source.hash(state);
  }
}

impl Debug for RspackRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RspackRegex")
      .field("flags", &self.flags)
      .field("source", &self.source)
      .finish()
  }
}

impl RspackRegex {
  #[inline]
  pub fn test(&self, text: &str) -> bool {
    self.algo.test(text)
  }

  #[inline]
  pub fn global(&self) -> bool {
    self.algo.global()
  }

  #[inline]
  pub fn sticky(&self) -> bool {
    self.algo.sticky()
  }

  #[inline]
  pub fn source(&self) -> &str {
    &self.source
  }

  #[inline]
  pub fn flags(&self) -> &str {
    &self.flags
  }

  #[inline]
  pub fn new(expr: &str) -> Result<Self, Error> {
    Self::with_flags(expr, "")
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

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/dependencies/ContextDependency.js#L30
  #[inline]
  pub fn to_source_string(&self) -> String {
    format!("/{}/{}", self.source, self.flags)
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/ContextModule.js#L192
  #[inline]
  pub fn to_pretty_string(&self, strip_slash: bool) -> String {
    if strip_slash {
      format!("{}{}", self.source, self.flags)
    } else {
      self.to_source_string()
    }
    .cow_replace('!', "%21")
    .cow_replace('|', "%7C")
    .into_owned()
  }

  pub fn try_from_swc_regex(ast: &Ast, value: SwcRegex) -> Result<Self, Error> {
    Self::with_flags(ast.get_utf8(value.exp(ast)), ast.get_utf8(value.flags(ast)))
  }
}

impl AsStringConverter for RspackRegex {
  fn to_string(&self) -> Result<String, rspack_cacheable::Error> {
    Ok(format!("{}#{}", self.flags, self.source))
  }
  fn from_str(s: &str) -> Result<Self, rspack_cacheable::Error>
  where
    Self: Sized,
  {
    let (flags, source) = s.split_once("#").expect("should have flags");
    Ok(RspackRegex::with_flags(source, flags).expect("should generate regex"))
  }
}
