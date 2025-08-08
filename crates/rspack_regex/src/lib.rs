mod napi;
mod native;
mod regress;

use std::fmt::Debug;

use cow_utils::CowUtils;
use native::RspackNativeRegex;
use regress::RspackRegressRegex;
use rspack_cacheable::{
  cacheable,
  with::{AsString, AsStringConverter},
};
use rspack_error::Error;
use swc_core::ecma::ast::Regex as SwcRegex;

#[derive(Debug, Clone)]
pub enum RspackRegexImpl {
  Native(RspackNativeRegex),
  Regress(RspackRegressRegex),
}

impl RspackRegexImpl {
  pub fn test(&self, text: &str) -> bool {
    match self {
      Self::Native(regex) => regex.test(text),
      Self::Regress(regex) => regex.test(text),
    }
  }
}

#[cacheable(with=AsString)]
#[derive(Debug, Clone)]
pub struct RspackRegex {
  pub regex: RspackRegexImpl,
  pub flags: String,
  pub source: String,
}

impl PartialEq for RspackRegex {
  fn eq(&self, other: &Self) -> bool {
    self.flags == other.flags && self.source == other.source && self.r#type() == other.r#type()
  }
}

impl Eq for RspackRegex {}

impl std::hash::Hash for RspackRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.flags.hash(state);
    self.source.hash(state);
    self.r#type().hash(state);
  }
}

impl RspackRegex {
  #[inline]
  pub fn r#type(&self) -> String {
    match self.regex {
      RspackRegexImpl::Native(_) => "native".to_string(),
      RspackRegexImpl::Regress(_) => "regress".to_string(),
    }
  }

  #[inline]
  pub fn test(&self, text: &str) -> bool {
    self.regex.test(text)
  }

  #[inline]
  pub fn global(&self) -> bool {
    self.flags.contains('g')
  }

  #[inline]
  pub fn sticky(&self) -> bool {
    self.flags.contains('y')
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
    match RspackNativeRegex::with_flags(expr, flags) {
      Ok(regex) => Ok(Self {
        regex: RspackRegexImpl::Native(regex),
        flags: flags.to_string(),
        source: expr.to_string(),
      }),
      Err(_) => {
        let regress = RspackRegressRegex::with_flags(expr, flags)?;
        Ok(Self {
          regex: RspackRegexImpl::Regress(regress),
          flags: flags.to_string(),
          source: expr.to_string(),
        })
      }
    }
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/dependencies/ContextDependency.js#L30
  #[inline]
  pub fn to_source_string(&self) -> String {
    format!("/{}/{}", self.source, self.flags)
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/ContextModule.js#L192
  #[inline]
  pub fn to_pretty_string(&self, strip_slash: bool) -> String {
    let res = if strip_slash {
      format!("{}{}", self.source, self.flags)
    } else {
      self.to_source_string()
    };

    res
      .cow_replace('!', "%21")
      .cow_replace('|', "%7C")
      .into_owned()
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

impl AsStringConverter for RspackRegex {
  fn to_string(&self) -> Result<String, rspack_cacheable::SerializeError> {
    Ok(format!("{}#{}", self.flags, self.source))
  }
  fn from_str(s: &str) -> Result<Self, rspack_cacheable::DeserializeError>
  where
    Self: Sized,
  {
    let (flags, source) = s.split_once("#").expect("should have flags");
    Ok(RspackRegex::with_flags(source, flags).expect("should generate regex"))
  }
}
