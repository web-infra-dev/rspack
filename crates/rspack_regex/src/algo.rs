use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use rspack_error::{internal_error, Error};

#[derive(Clone, Debug)]
pub enum Algo {
  /// See details at https://github.com/web-infra-dev/rspack/pull/3113
  EndWith { pats: Vec<String> },
  /// Regress is considered having the same behaviors as RegExp in JS.
  Regress(regress::Regex),
}

impl Algo {
  pub fn new(expr: &str, flags: &str) -> Result<Algo, Error> {
    if let Some(algo) = Self::try_compile_to_end_with_fast_path(expr) {
      Ok(algo)
    } else {
      regress::Regex::with_flags(expr, flags)
        .map(Algo::Regress)
        .map_err(|err| {
          internal_error!("Can't construct regex `/{expr}/{flags}`, original error message: {err}")
        })
    }
  }

  pub fn try_compile_to_end_with_fast_path(expr: &str) -> Option<Algo> {
    let hir = regex_syntax::parse(expr).ok()?;
    let seq = regex_syntax::hir::literal::Extractor::new()
      .kind(ExtractKind::Suffix)
      .extract(&hir);
    if is_ends_with_regex(&hir) && seq.is_exact() {
      let pats = seq
        .literals()?
        .iter()
        .map(|item| String::from_utf8_lossy(item.as_bytes()).to_string())
        .collect::<Vec<_>>();

      Some(Algo::EndWith { pats })
    } else {
      None
    }
  }

  pub(crate) fn test(&self, str: &str) -> bool {
    match self {
      Algo::Regress(regex) => regex.find(str).is_some(),
      Algo::EndWith { pats } => pats.iter().any(|pat| str.ends_with(pat)),
    }
  }

  #[cfg(test)]
  fn is_end_with(&self) -> bool {
    matches!(self, Self::EndWith { .. })
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
    assert!(Algo::new("\\.js$", "").unwrap().is_end_with());
    assert!(Algo::new("\\.(jsx?|tsx?)$", "").unwrap().is_end_with());
    assert!(Algo::new("\\.(svg|png)$", "").unwrap().is_end_with());
  }

  #[test]
  fn check_slow_path() {
    // this is a full match
    assert!(Algo::new("^\\.(svg|png)$", "").unwrap().is_regress());
    // wildcard match
    assert!(Algo::new("\\..(svg|png)$", "").unwrap().is_regress());
  }
}
