use std::time::Instant;

use aho_corasick::{AhoCorasick, PatternID};
use regex_syntax::ast::Concat;
use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use rspack_error::Result;

fn main() {
  let str = "\\.(jsx?|tsx?)$";
  let test_str = "fejaifjeioajifjeawifjioeawjiofjawijiojjfieoajfiowaejoijfoiewajijfwea.svg";
  let count = 10000;

  let start = Instant::now();
  let re = Regex2::new(str).unwrap();
  for i in 0..count {
    re.test(test_str);
  }
  dbg!(*&start.elapsed());

  let start = Instant::now();
  let re = Regex1::new(str).unwrap();
  for i in 0..count {
    re.test(test_str);
  }
  dbg!(*&start.elapsed());
  let start = Instant::now();
  let reg = regress::Regex::new(str).unwrap();
  for i in 0..count {
    reg.find(test_str).is_some();
  }
  dbg!(&start.elapsed());
}

enum Regex1 {
  FastReg(regex::Regex),
  SlowReg(regress::Regex),
}

impl Regex1 {
  pub fn new(str: &str) -> Result<Regex1> {
    match regex::Regex::new(str) {
      Ok(reg) => Ok(Regex1::FastReg(reg)),
      Err(_) => Ok(regress::Regex::new(str).map(Regex1::SlowReg).unwrap()),
    }
  }

  pub fn test(&self, str: &str) -> bool {
    match self {
      Regex1::FastReg(reg) => reg.is_match(str),
      Regex1::SlowReg(reg) => reg.find(str).is_some(),
    }
  }
}

enum Regex2 {
  FastCustom(Box<dyn Fn(&str) -> bool>),
  SlowReg(regress::Regex),
}

impl Regex2 {
  pub fn new(str: &str) -> Result<Regex2> {
    match regex_syntax::parse(str) {
      Ok(ast) => {
        let hir = regex_syntax::parse(str).unwrap();
        let res = regex_syntax::hir::literal::Extractor::new()
          .kind(ExtractKind::Suffix)
          .extract(&hir);
        if res.is_exact() {
          // let set = RegexSet::new(
          //   res
          //     .literals()
          //     .unwrap()
          //     .iter()
          //     .map(|item| std::str::from_utf8(item.as_bytes()).unwrap().to_string() + "$"),
          // )
          // .unwrap();
          //
          let string_list = res
            .literals()
            .unwrap()
            .iter()
            .map(|item| {
              let s = std::str::from_utf8(item.as_bytes()).unwrap();
              s.to_string()
            })
            .collect::<Vec<_>>();

          Ok(Regex2::FastCustom(Box::new(move |str| {
            string_list.iter().any(|item| str.ends_with(item))
          })))
        } else {
          Ok(regress::Regex::new(str).map(Regex2::SlowReg).unwrap())
        }
      }
      Err(_) => Ok(regress::Regex::new(str).map(Regex2::SlowReg).unwrap()),
    }
  }

  pub fn test(&self, str: &str) -> bool {
    match self {
      Regex2::FastCustom(fast) => (fast)(str),
      Regex2::SlowReg(reg) => reg.find(str).is_some(),
    }
  }
}

pub fn convert_hir_to_pattern<'a>(hir: &'a HirKind, str_list: &mut Vec<&'a str>) {
  match hir {
    HirKind::Empty => {}
    HirKind::Literal(lit) => match std::str::from_utf8(lit.0.as_ref()) {
      Ok(str) => {
        str_list.push(str);
      }
      Err(_) => {}
    },
    HirKind::Class(_) => {}

    HirKind::Concat(hirs) if hirs.len() == 2 => match (hirs[0].kind(), hirs[1].kind()) {
      (HirKind::Literal(lit), HirKind::Look(Look::End)) => {}
      _ => {}
    },
    _ => {}
  }
}
