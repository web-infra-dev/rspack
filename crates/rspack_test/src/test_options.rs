use rspack_binding_options::{normalize_bundle_options, RawOptions};
use rspack_core::CompilerOptions;

use std::{collections::HashMap, path::Path};

pub trait RawOptionsExt {
  fn from_fixture(fixture_path: &Path) -> Self;
  fn to_compiler_options(self) -> CompilerOptions;
}

impl RawOptionsExt for RawOptions {
  fn from_fixture(fixture_path: &Path) -> Self {
    let pkg_path = fixture_path.join("test.config.js");
    let mut options = if pkg_path.exists() {
      let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
      const HEAD: &str = "var module = {exports: {}};";
      const TAIL: &str = "JSON.stringify(module.exports)";
      let qjs_context = quick_js::Context::new().unwrap();
      let value = qjs_context
        .eval(&format!("{HEAD}\n{pkg_content}\n{TAIL}"))
        .unwrap();
      let options: RawOptions = serde_json::from_str(&value.into_string().unwrap()).unwrap();
      options
    } else {
      RawOptions {
        entry: Some(HashMap::from([(
          "main".to_string(),
          fixture_path.join("index.js").to_str().unwrap().to_string(),
        )])),
        ..Default::default()
      }
    };
    assert!(
      options.context.is_none(),
      "You should not specify `root` in config. It would probably resolve to a wrong path"
    );
    options.context = Some(fixture_path.to_str().unwrap().to_string());

    // set builtins.minify default to false
    if options.builtins.is_none() {
      options.builtins = Some(Default::default())
    };
    if let Some(ref mut raw_builtins) = options.builtins && raw_builtins.minify.is_none() {
        raw_builtins.minify = Some(false);
    };

    // target default set es_version to es2022
    if options.target.is_none() {
      options.target = Some(vec!["web".to_string(), "es2022".to_string()]);
    };
    if let Some(ref mut target) = options.target && !target.iter().any(|i| i.as_str().starts_with("es")) {
      target.push("es2022".to_string());
    };
    options
  }

  fn to_compiler_options(self) -> CompilerOptions {
    normalize_bundle_options(self).unwrap()
  }
}

pub fn read_test_config_and_normalize(fixture_path: &Path) -> CompilerOptions {
  let options = RawOptions::from_fixture(fixture_path);
  options.to_compiler_options()
}
