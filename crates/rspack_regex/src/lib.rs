use std::sync::Arc;

use regex_syntax::ast::Concat;
use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use regress::{Match, Matches, Regex};
use rspack_error::{internal_error, Error};
use swc_core::ecma::ast::Regex as SwcRegex;
use swc_core::ecma::visit::visit_mut_ts_interface_body;

/// Using wrapper type required by [TryFrom] trait
#[derive(Debug, Clone)]
pub struct RspackRegex {
  algo: Algo,
}

#[derive(Clone)]
pub(crate) enum Algo {
  FastCustom(Arc<dyn Fn(&str) -> bool + Send + Sync>),
  // FastCustom(bool),
  Regress(regress::Regex),
}

impl std::fmt::Debug for Algo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::FastCustom(arg0) => write!(f, "FastCustom(...)"),
      Self::Regress(arg0) => f.debug_tuple("Regress").field(arg0).finish(),
    }
  }
}

impl Algo {
  pub fn new(expr: &str, flags: &str) -> Result<Algo, Error> {
    match regex_syntax::parse(expr) {
      Ok(hir) => {
        let seq = regex_syntax::hir::literal::Extractor::new()
          .kind(ExtractKind::Suffix)
          .extract(&hir);
        if seq.is_exact() {
          let string_list = seq
            .literals()
            .unwrap()
            .iter()
            .map(|item| String::from_utf8_lossy(item.as_bytes()).to_string())
            .collect::<Vec<_>>();

          // Ok(Algo::FastCustom(false))
          Ok(Algo::FastCustom(Arc::new(move |str: &str| {
            string_list.iter().any(|item| str.ends_with(item))
          })))
        } else {
          regress::Regex::with_flags(expr, flags)
            .map(Algo::Regress)
            .map_err(|err| {
              internal_error!(
                "Can't construct regex `/{expr}/{flags}`, original error message: {err}"
              )
            })
        }
      }
      // fallback to regress:regex
      Err(_) => regress::Regex::with_flags(expr, flags)
        .map(Algo::Regress)
        .map_err(|err| {
          internal_error!("Can't construct regex `/{expr}/{flags}`, original error message: {err}")
        }),
    }
  }
}

impl RspackRegex {
  /// # Panic
  /// [Algo::FastCustom] does not implement `find`, this method may panic if original
  /// regex could be optimized by fast path
  pub fn find(&self, text: &str) -> Option<Match> {
    match &self.algo {
      Algo::FastCustom(_) => panic!("Algo::FastCustom does not implement `find`"),
      Algo::Regress(regex) => regex.find(text),
    }
  }

  pub fn test(&self, text: &str) -> bool {
    match &self.algo {
      // Algo::FastCustom(fast) => fast(text),
      Algo::FastCustom(_) => panic!(),
      Algo::Regress(regex) => regex.find(text).is_some(),
    }
  }

  /// # Panic
  /// [Algo::FastCustom] does not implement `find_iter`, this method may panic if original
  /// regex could be optimized by fast path
  pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> Matches<'r, 't> {
    match &self.algo {
      Algo::FastCustom(_) => panic!("Algo::FastCustom does not implement `find_iter`"),
      Algo::Regress(regex) => regex.find_iter(text),
    }
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

  pub fn new_with_optimized(expr: &str, flags: &str) -> Result<Self, Error> {
    Algo::new(expr, flags).map(|algo| RspackRegex { algo })
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
