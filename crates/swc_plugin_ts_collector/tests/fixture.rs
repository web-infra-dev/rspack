use std::{
  fs,
  path::{Path, PathBuf},
  rc::Rc,
  sync::Arc,
};

use glob::glob;
use rspack_javascript_compiler::{JavaScriptCompiler, transform::SwcOptions};
use rspack_swc_plugin_ts_collector::{ExportedEnumCollector, TypeExportsCollector};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  common::{FileName, SyntaxContext, comments::SingleThreadedComments},
  ecma::{
    ast::noop_pass,
    parser::{Syntax, TsSyntax},
    visit::VisitWith,
  },
};

fn snapshot_name(root: &Path, input: &Path) -> String {
  #[allow(clippy::disallowed_methods)]
  input
    .strip_prefix(root)
    .expect("input should under tests dir")
    .with_extension("")
    .to_string_lossy()
    .replace(['/', '\\'], "__")
}

#[test]
fn type_exports() {
  let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string()).join("tests");
  let cases = glob(&format!("{}/type-exports/**/*.ts", tests_dir.display()))
    .expect("glob failed")
    .collect::<Result<Vec<_>, _>>()
    .expect("glob error");
  assert!(!cases.is_empty(), "no test cases found");
  for input in cases {
    let snapshot_name = snapshot_name(&tests_dir, &input);
    let source = fs::read_to_string(&input).expect("failed to read input.ts");
    let compiler = JavaScriptCompiler::new();
    let mut options = SwcOptions::default();
    options.config.jsc.syntax = Some(Syntax::Typescript(TsSyntax::default()));
    let mut type_exports_results = FxHashSet::default();
    let comments = Rc::new(SingleThreadedComments::default());
    let _ = compiler
      .transform(
        source,
        Some(Arc::new(FileName::Real(input))),
        comments,
        options,
        None,
        |program, _| {
          program.visit_with(&mut TypeExportsCollector::new(&mut type_exports_results));
        },
        |_| noop_pass(),
      )
      .expect("input.ts should be valid typescript");
    insta::assert_debug_snapshot!(snapshot_name, type_exports_results);
  }
}

#[test]
fn enums() {
  let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string()).join("tests");
  let cases = glob(&format!("{}/enums/**/*.ts", tests_dir.display()))
    .expect("glob failed")
    .collect::<Result<Vec<_>, _>>()
    .expect("glob error");
  assert!(!cases.is_empty(), "no test cases found");
  for input in cases {
    let snapshot_name = snapshot_name(&tests_dir, &input);
    let source = fs::read_to_string(&input).expect("failed to read input.ts");
    let compiler = JavaScriptCompiler::new();
    let mut options = SwcOptions::default();
    options.config.jsc.syntax = Some(Syntax::Typescript(TsSyntax::default()));
    let mut enum_results = FxHashMap::default();
    let comments = Rc::new(SingleThreadedComments::default());
    let _ = compiler
      .transform(
        source,
        Some(Arc::new(FileName::Real(input))),
        comments,
        options,
        None,
        |program, _| {
          program.visit_with(&mut ExportedEnumCollector::new(
            false,
            SyntaxContext::empty(),
            &mut enum_results,
          ));
        },
        |_| noop_pass(),
      )
      .expect("input.ts should be valid typescript");
    insta::assert_debug_snapshot!(snapshot_name, enum_results);
  }
}
