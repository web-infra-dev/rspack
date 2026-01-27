use std::{fs, path::PathBuf};

use glob::glob;
use rspack_javascript_compiler::{JavaScriptCompiler, transform::SwcOptions};
use rspack_swc_plugin_ts_collector::TypeExportsCollector;
use rustc_hash::FxHashSet;
use swc_core::{
  atoms::Atom,
  common::FileName,
  ecma::{
    ast::noop_pass,
    parser::{Syntax, TsSyntax},
    visit::VisitWith,
  },
};

#[test]
fn type_exports() {
  let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string()).join("tests");
  let cases = glob(&format!("{}/type-exports/**/input.ts", tests_dir.display()))
    .expect("glob failed")
    .collect::<Result<Vec<_>, _>>()
    .expect("glob error");
  assert!(!cases.is_empty(), "no test cases found");
  for input in cases {
    let info = input
      .parent()
      .expect("test should under tests dir")
      .join("info.json");
    let info = fs::read_to_string(info).expect("failed to read info.json");
    let info: Vec<String> = serde_json::from_str(&info).expect("info.json is not a valid json");
    let source = fs::read_to_string(&input).expect("failed to read input.ts");
    let compiler = JavaScriptCompiler::new();
    let mut options = SwcOptions::default();
    options.config.jsc.syntax = Some(Syntax::Typescript(TsSyntax::default()));
    let mut type_exports_results = FxHashSet::default();
    let _ = compiler
      .transform(
        source,
        Some(FileName::Real(input)),
        options,
        None,
        |program, _| {
          program.visit_with(&mut TypeExportsCollector::new(&mut type_exports_results));
        },
        |_| noop_pass(),
      )
      .expect("input.ts should be valid typescript");
    assert_eq!(
      info.len(),
      type_exports_results.len(),
      "expected: {info:#?}\nactual: {type_exports_results:#?}"
    );
    for e in &info {
      assert!(
        type_exports_results.contains(&Atom::from(e.as_str())),
        "expected: {info:#?}\nactual: {type_exports_results:#?}"
      );
    }
  }
}
