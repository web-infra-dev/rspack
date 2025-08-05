mod algo;
mod napi;

use std::fmt::Debug;

use cow_utils::CowUtils;
use rspack_cacheable::{
  cacheable,
  with::{AsString, AsStringConverter},
};
use rspack_error::{Error, error};
use swc_core::ecma::ast::Regex as SwcRegex;

use self::algo::Algo;

#[cacheable(with=AsString)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RspackRegex {
  Regex(RspackNativeRegex),
  Regress(RspackRegressRegex),
}

impl RspackRegex {
  #[inline]
  pub fn test(&self, text: &str) -> bool {
    match self {
      Self::Regex(regex) => regex.regex.is_match(text),
      Self::Regress(regress) => regress.algo.test(text),
    }
  }

  #[inline]
  pub fn global(&self) -> bool {
    match self {
      // return false for native regex otherwise context options will emit warning
      // but it is safe to do so because we can not use regex to capture multiple matches
      Self::Regex(regex) => regex.flags.contains('g'),
      Self::Regress(regress) => regress.algo.global(),
    }
  }

  #[inline]
  pub fn sticky(&self) -> bool {
    match self {
      Self::Regex(regex) => regex.flags.contains('y'),
      Self::Regress(regress) => regress.algo.sticky(),
    }
  }

  #[inline]
  pub fn source(&self) -> &str {
    match self {
      Self::Regex(regex) => &regex.source,
      Self::Regress(regress) => &regress.source,
    }
  }

  #[inline]
  pub fn flags(&self) -> &str {
    match self {
      Self::Regex(regex) => &regex.flags,
      Self::Regress(regress) => &regress.flags,
    }
  }

  #[inline]
  pub fn new(expr: &str) -> Result<Self, Error> {
    match RspackNativeRegex::with_flags(expr, "") {
      Ok(regex) => Ok(Self::Regex(regex)),
      Err(e) => {
        println!("create native regex failed: {expr} {e}");
        let regress = RspackRegressRegex::with_flags(expr, "")?;
        Ok(Self::Regress(regress))
      }
    }
  }

  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    match RspackNativeRegex::with_flags(expr, flags) {
      Ok(regex) => Ok(Self::Regex(regex)),
      Err(e) => {
        println!("create native regex failed: {expr} with {flags} {e}");
        let regress = RspackRegressRegex::with_flags(expr, flags)?;
        Ok(Self::Regress(regress))
      }
    }
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/dependencies/ContextDependency.js#L30
  #[inline]
  pub fn to_source_string(&self) -> String {
    match self {
      Self::Regex(regex) => format!("/{}/{}", regex.source, regex.flags),
      Self::Regress(regress) => format!("/{}/{}", regress.source, regress.flags),
    }
  }

  // https://github.com/webpack/webpack/blob/4baf1c075d59babd028f8201526cb8c4acfd24a0/lib/ContextModule.js#L192
  #[inline]
  pub fn to_pretty_string(&self, strip_slash: bool) -> String {
    let res = if strip_slash {
      match self {
        Self::Regex(regex) => format!("{}{}", regex.source, regex.flags),
        Self::Regress(regress) => format!("{}{}", regress.source, regress.flags),
      }
    } else {
      self.to_source_string()
    };

    res
      .cow_replace('!', "%21")
      .cow_replace('|', "%7C")
      .into_owned()
  }
}

#[derive(Clone, Debug)]
pub struct RspackNativeRegex {
  regex: regex::Regex,
  flags: String,
  source: String,
}

impl std::hash::Hash for RspackNativeRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.flags.hash(state);
    self.source.hash(state);
  }
}

impl PartialEq for RspackNativeRegex {
  fn eq(&self, other: &Self) -> bool {
    self.flags == other.flags && self.source == other.source
  }
}

impl Eq for RspackNativeRegex {}

impl RspackNativeRegex {
  pub fn with_flags(expr: &str, raw_flags: &str) -> Result<Self, Error> {
    let pattern = expr.replace("\\/", "/");

    let mut flags = raw_flags.chars().collect::<Vec<char>>();
    flags.sort_unstable();
    let mut applied_flags = String::new();
    // https://github.com/vercel/next.js/blob/203adbd5d054609812d1f3666184875dcca13f3a/turbopack/crates/turbo-esregex/src/lib.rs#L71-L94
    for flag in &flags {
      match flag {
        // indices for substring matches: not relevant for the regex itself
        'd' => {}
        // global: default in rust, ignore
        'g' => {}
        // case-insensitive: letters match both upper and lower case
        'i' => applied_flags.push('i'),
        // multi-line mode: ^ and $ match begin/end of line
        'm' => applied_flags.push('m'),
        // allow . to match \n
        's' => applied_flags.push('s'),
        // Unicode support (enabled by default)
        'u' => applied_flags.push('u'),
        // sticky search: not relevant for the regex itself
        'y' => {}
        _ => {
          return Err(error!(
            "unsupported flag `{flag}` in regex: `{pattern}` with flags: `{raw_flags}`"
          ));
        }
      }
    }

    let regex = if applied_flags.is_empty() {
      regex::Regex::new(&pattern).map_err(|e| error!(e))?
    } else {
      regex::Regex::new(&format!("(?{applied_flags}){pattern}")).map_err(|e| error!(e))?
    };

    Ok(Self {
      regex,
      flags: flags.into_iter().collect::<String>(),
      source: expr.to_string(),
    })
  }
}

#[derive(Clone)]
pub struct RspackRegressRegex {
  algo: Box<Algo>,
  pub flags: String,
  pub source: String,
}

impl PartialEq for RspackRegressRegex {
  fn eq(&self, other: &Self) -> bool {
    self.flags == other.flags && self.source == other.source
  }
}

impl Eq for RspackRegressRegex {}

impl std::hash::Hash for RspackRegressRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.flags.hash(state);
    self.source.hash(state);
  }
}

impl Debug for RspackRegressRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RspackRegressRegex")
      .field("flags", &self.flags)
      .field("source", &self.source)
      .finish()
  }
}

impl RspackRegressRegex {
  pub fn with_flags(expr: &str, flags: &str) -> Result<Self, Error> {
    let mut chars = flags.chars().collect::<Vec<char>>();
    chars.sort_unstable();
    Ok(Self {
      flags: chars.into_iter().collect::<String>(),
      source: expr.to_string(),
      algo: Box::new(Algo::new(expr, flags)?),
    })
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
    match self {
      Self::Regex(regex) => Ok(format!("{}#{}", regex.flags, regex.source)),
      Self::Regress(regress) => Ok(format!("{}#{}", regress.flags, regress.source)),
    }
  }
  fn from_str(s: &str) -> Result<Self, rspack_cacheable::DeserializeError>
  where
    Self: Sized,
  {
    let (flags, source) = s.split_once("#").expect("should have flags");
    Ok(RspackRegex::with_flags(source, flags).expect("should generate regex"))
  }
}
