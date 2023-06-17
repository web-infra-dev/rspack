use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use rspack_error::{internal_error, Error};

#[derive(Clone, Debug)]
pub enum Algo {
  /// Regress is considered having the same behaviors as RegExp in JS.
  /// But Regress has poor performance. To improve performance of regex matching,
  /// we would try to use some fast algo to do matching, when we detect some special pattern.
  /// See details at https://github.com/web-infra-dev/rspack/pull/3113
  EndWith {
    pats: Vec<String>,
  },
  Regress(regress::Regex),
}

impl Algo {
  pub(crate) fn new(expr: &str, flags: &str) -> Result<Algo, Error> {
    let ignore_case = flags.contains('i');
    if let Some(algo) = Self::try_compile_to_end_with_fast_path(expr) && !ignore_case {
      Ok(algo)
    } else {
      regress::Regex::with_flags(expr, flags)
        .map(Algo::Regress)
        .map_err(|err| {
          internal_error!("Can't construct regex `/{expr}/{flags}`, original error message: {err}")
        })
    }
  }

  fn try_compile_to_end_with_fast_path(expr: &str) -> Option<Algo> {
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
}

#[cfg(test)]
impl Algo {
  fn end_with_pats(&self) -> std::collections::HashSet<&str> {
    match self {
      Algo::EndWith { pats } => pats.iter().map(|s| s.as_str()).collect(),
      Algo::Regress(_) => panic!("expect EndWith"),
    }
  }

  fn is_end_with(&self) -> bool {
    matches!(self, Self::EndWith { .. })
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
  fn should_use_end_with_algo_with_i_flag() {
    assert!(Algo::new("\\.js$", "").unwrap().is_end_with());
    assert!(!Algo::new("\\.js$", "i").unwrap().is_end_with());
  }

  #[test]
  fn correct_end_with() {
    use std::collections::HashSet;
    let algo = Algo::new("\\.js$", "").unwrap();
    assert_eq!(algo.end_with_pats(), HashSet::from([".js"]));
    let algo = Algo::new("\\.(jsx?|tsx?)$", "").unwrap();
    assert_eq!(
      algo.end_with_pats(),
      HashSet::from([".jsx", ".tsx", ".js", ".ts"])
    );
    let algo = Algo::new("\\.(svg|png)$", "").unwrap();
    assert_eq!(algo.end_with_pats(), HashSet::from([".svg", ".png"]));
  }

  #[test]
  fn check_slow_path() {
    // this is a full match
    assert!(Algo::new("^\\.(svg|png)$", "").unwrap().is_regress());
    // wildcard match
    assert!(Algo::new("\\..(svg|png)$", "").unwrap().is_regress());
  }
}
