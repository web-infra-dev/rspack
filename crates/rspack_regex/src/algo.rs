use std::fmt::Debug;
use std::hash::Hash;

use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use regress::Match;
use rspack_error::{error, Error};

#[derive(Clone)]
pub struct HashRegressRegex {
  pub regex: regress::Regex,
  expr: String,
  flags: String,
}

impl Hash for HashRegressRegex {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.expr.hash(state);
    self.flags.hash(state)
  }
}

impl Debug for HashRegressRegex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(&self.regex, f)
  }
}

impl HashRegressRegex {
  pub(crate) fn new(expr: &str, flags: &str) -> Result<Self, Error> {
    match regress::Regex::with_flags(expr, flags) {
      Ok(regex) => Ok(Self {
        regex,
        expr: expr.to_string(),
        flags: flags.to_string(),
      }),
      Err(err) => Err(error!(
        "Can't construct regex `/{expr}/{flags}`, original error message: {err}"
      )),
    }
  }

  fn find(&self, text: &str) -> Option<Match> {
    self.regex.find(text)
  }
}

#[derive(Clone, Debug, Hash)]
pub enum Algo {
  /// Regress is considered having the same behaviors as RegExp in JS.
  /// But Regress has poor performance. To improve performance of regex matching,
  /// we would try to use some fast algo to do matching, when we detect some special pattern.
  /// See details at https://github.com/web-infra-dev/rspack/pull/3113
  EndWith {
    pats: Vec<String>,
  },
  Regress(HashRegressRegex),
}

impl Algo {
  pub(crate) fn new(expr: &str, flags: &str) -> Result<Algo, Error> {
    let ignore_case = flags.contains('i') || flags.contains('g') || flags.contains('y');
    if let Some(algo) = Self::try_compile_to_end_with_fast_path(expr)
      && !ignore_case
    {
      Ok(algo)
    } else {
      match HashRegressRegex::new(expr, flags) {
        Ok(regex) => Ok(Algo::Regress(regex)),
        Err(e) => Err(e),
      }
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

  pub(crate) fn global(&self) -> bool {
    match self {
      Algo::Regress(reg) => reg.flags.contains('g'),
      Algo::EndWith { .. } => false,
    }
  }

  pub(crate) fn sticky(&self) -> bool {
    match self {
      Algo::Regress(reg) => reg.flags.contains('y'),
      Algo::EndWith { .. } => false,
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
