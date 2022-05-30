mod utils;

use rspack::bundler::BundleOptions;
use utils::{compile, compile_with_options};

use rspack_core::Loader;

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::Path;

#[test]
fn single_entry() {
  compile("single-entry", vec![]);
}

#[test]
fn multi_entry() {
  compile("multi-entry", vec![]);
}

#[test]
fn cycle_dep() {
  compile("cycle-dep", vec![]);
}

#[test]
fn dynamic_import() {
  compile("dynamic-import", vec![]);
}

#[test]
#[ignore = "not support npm yet"]
fn npm() {
  compile("npm", vec![]);
}

#[test]
fn cjs() {
  compile("cjs", vec![]);
}

#[test]
fn css_bundle_test() {
  compile_with_options(
    "css",
    BundleOptions {
      loader: HashMap::from_iter([
        ("css".to_string(), Loader::Css),
        ("less".to_string(), Loader::Less),
        ("sass".to_string(), Loader::Sass),
        ("scss".to_string(), Loader::Sass),
        ("svg".to_string(), Loader::DataURI),
      ]),
      ..Default::default()
    },
    vec![],
  );

  pub fn path_resolve(path: &str) -> String {
    let work_cwd = env!("CARGO_MANIFEST_DIR");
    let os_work_cwd = OsString::from(work_cwd);
    Path::new(&os_work_cwd)
      .join(path)
      .into_os_string()
      .into_string()
      .unwrap()
  }

  let _dist_css_file1 = path_resolve("fixtures/css/dist/index.css");
  let _dist_css_file2 = path_resolve("fixtures/css/dist/liba.css");
  // FIXME: The output filename of chunk is not stable now, should not rely on it.
  // assert_eq!(Path::new(dist_css_file1.as_str()).exists(), true);
  // assert_eq!(Path::new(dist_css_file2.as_str()).exists(), true);
}

#[test]
fn basic_ts() {
  compile("basic-ts", vec![]);
}

#[test]
fn svgr() {
  compile("svgr", vec![]);
}

#[test]
fn splitting() {
  compile("code-splitting", vec![]);
}

#[test]
fn loader() {
  compile_with_options(
    "loader",
    BundleOptions {
      loader: vec![
        ("svg".to_string(), Loader::DataURI),
        // Json is supported by default
        // ("json".to_string(), Loader::Json),
        ("txt".to_string(), Loader::Text),
      ]
      .into_iter()
      .collect(),
      ..Default::default()
    },
    vec![],
  );
}

#[test]
fn stack_overflow_mockjs() {
  compile("stack_overflow_mockjs", vec![]);
}
