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
    match regex_syntax::parse(expr) {
      Ok(hir) => {
        let seq = regex_syntax::hir::literal::Extractor::new()
          .kind(ExtractKind::Suffix)
          .extract(&hir);
        if is_ends_with_regex(&hir) && seq.is_exact() {
          let string_list = seq
            .literals()
            .unwrap()
            .iter()
            .map(|item| String::from_utf8_lossy(item.as_bytes()).to_string())
            .collect::<Vec<_>>();

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

  pub(crate) fn test(&self, str: &str) -> bool {
    match self {
      Algo::FastCustom(fast) => fast(str),
      Algo::Regress(regex) => regex.find(str).is_some(),
    }
  }

  fn is_fast_custom(&self) -> bool {
    matches!(self, Self::FastCustom(..))
  }

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
