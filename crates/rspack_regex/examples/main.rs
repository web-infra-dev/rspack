use std::time::Instant;

use aho_corasick::{AhoCorasick, PatternID};
use regex_syntax::ast::Concat;
use regex_syntax::hir::literal::ExtractKind;
use regex_syntax::hir::{Hir, HirKind, Look};
use rspack_error::Result;

fn main() {
  let string = include_str!("../result.log");
  let string_list = string.split("\n").collect::<Vec<_>>();
  let count = 6;
  let mut reg_list = vec![
    "\\.json$",
    "\\.mjs$",
    "\\.js$",
    "\\.cjs$",
    "\\.js$",
    "\\.jsx$",
    "\\.ts$",
    "\\.tsx$",
    "\\.module\\.css$",
    "\\.css$",
    "\\.less$",
    "\\.module\\.less$",
    "\\.svg$",
    "\\.png$",
  ];

  let test_str = "";
  let total_count = count * string_list.len();
  let start = Instant::now();
  let re_list = reg_list
    .iter()
    .map(|item| Regex2::new(item).unwrap())
    .collect::<Vec<_>>();
  for re in re_list.iter() {
    for i in 0..count {
      for test_str in string_list.iter() {
        re.test(test_str);
      }
    }
  }
  println!(
    "{}  {} module:\n{:?}",
    "custom search",
    total_count,
    start.elapsed()
  );
  let start = Instant::now();
  let re_list = reg_list
    .iter()
    .map(|item| Regex1::new(item).unwrap())
    .collect::<Vec<_>>();
  for re in re_list.iter() {
    for i in 0..count {
      for test_str in string_list.iter() {
        re.test(test_str);
      }
    }
  }
  println!(
    "{}  {} module:\n{:?}",
    "regex",
    total_count,
    start.elapsed()
  );
  let start = Instant::now();

  let re_list = reg_list
    .iter()
    .map(|item| regress::Regex::new(item).unwrap())
    .collect::<Vec<_>>();
  for re in re_list.iter() {
    for i in 0..count {
      for test_str in string_list.iter() {
        _ = re.find(test_str).is_some();
      }
    }
  }

  println!(
    "{}  {} module:\n{:?}",
    "regress",
    total_count,
    start.elapsed()
  );
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
      Regex2::FastCustom(fast) => fast(str),
      Regex2::SlowReg(reg) => reg.find(str).is_some(),
    }
  }
}
