use std::sync::Arc;

use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use rspack_error::{internal_error, Error};

#[derive(Clone)]
pub enum Algo {
  FastCustom(Arc<dyn Fn(&str) -> bool + Send + Sync>),
  Regress(regress::Regex),
}

impl std::fmt::Debug for Algo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::FastCustom(_) => write!(f, "FastCustom(...)"),
      Self::Regress(arg0) => f.debug_tuple("Regress").field(arg0).finish(),
    }
  }
}

impl Algo {
  pub fn new(expr: &str, flags: &str) -> Result<Algo, Error> {
    if let Some(algo) = Self::try_fast_path(expr) {
      Ok(algo)
    } else {
      regress::Regex::with_flags(expr, flags)
        .map(Algo::Regress)
        .map_err(|err| {
          internal_error!("Can't construct regex `/{expr}/{flags}`, original error message: {err}")
        })
    }
  }

  pub fn try_fast_path(expr: &str) -> Option<Algo> {
    let hir = regex_syntax::parse(expr).ok()?;
    let seq = regex_syntax::hir::literal::Extractor::new()
      .kind(ExtractKind::Suffix)
      .extract(&hir);
    if is_ends_with_regex(&hir) && seq.is_exact() {
      let string_list = seq
        .literals()?
        .iter()
        .map(|item| String::from_utf8_lossy(item.as_bytes()).to_string())
        .collect::<Vec<_>>();

      Some(Algo::FastCustom(Arc::new(move |str: &str| {
        string_list.iter().any(|item| str.ends_with(item))
      })))
    } else {
      None
    }
  }

  pub(crate) fn test(&self, str: &str) -> bool {
    match self {
      Algo::FastCustom(fast) => fast(str),
      Algo::Regress(regex) => regex.find(str).is_some(),
    }
  }

  #[cfg(test)]
  fn is_fast_custom(&self) -> bool {
    matches!(self, Self::FastCustom(..))
  }

  #[cfg(test)]
  fn is_regress(&self) -> bool {
    matches!(self, Self::Regress(..))
  }
}

fn is_ends_with_regex(hir: &Hir) -> bool {
  if let HirKind::Concat(list) = hir.kind() {
    list[0].kind() != &HirKind::Look(Look::Start)
      && list[list.len() - 1].kind() == &HirKind::Look(Look::End)
  } else {
    false
  }
}

#[cfg(test)]
mod test_algo {
  use super::*;
  #[test]
  fn check_fast_path() {
    assert!(Algo::new("\\.js$", "").unwrap().is_fast_custom());
    assert!(Algo::new("\\.(jsx?|tsx?)$", "").unwrap().is_fast_custom());
    assert!(Algo::new("\\.(svg|png)$", "").unwrap().is_fast_custom());
  }

  #[test]
  fn check_slow_path() {
    // this is a full match
    assert!(Algo::new("^\\.(svg|png)$", "").unwrap().is_regress());
    // wildcard match
    assert!(Algo::new("\\..(svg|png)$", "").unwrap().is_regress());
  }
}
