use std::path::{Path, PathBuf};

use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use rspack_binding_options::{JsLoaderRunner, RawOptions, RawOptionsApply};
use rspack_core::{BoxPlugin, Compiler, CompilerOptions};
use rspack_fs::AsyncNativeFileSystem;
use rspack_tracing::enable_tracing_by_env;

use crate::{eval_raw::evaluate_to_json, test_config::TestConfig};

pub fn apply_from_fixture(fixture_path: &Path) -> (CompilerOptions, Vec<BoxPlugin>) {
  let js_config = fixture_path.join("test.config.js");
  if js_config.exists() {
    let raw = evaluate_to_json(&js_config);
    let raw: RawOptions = serde_json::from_slice(&raw).expect("ok");
    let mut plugins = Vec::new();
    let compiler_options = raw
      .apply(&mut plugins, &JsLoaderRunner::noop())
      .expect("should be ok");
    return (compiler_options, plugins);
  }
  let json_config = fixture_path.join("test.config.json");
  let test_config = TestConfig::from_config_path(&json_config);
  test_config.apply(fixture_path.to_path_buf())
}

#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) -> Compiler<AsyncNativeFileSystem> {
  enable_tracing_by_env();

  let (mut options, plugins) = apply_from_fixture(fixture_path);
  for (_, entry) in options.entry.iter_mut() {
    entry.runtime = Some("runtime".to_string());
  }
  // clean output
  if options.output.path.exists() {
    std::fs::remove_dir_all(&options.output.path).expect("should remove output");
  }
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);
  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixtrue {fixture_path:?}, {e:#?}"));
  let stats = compiler.compilation.get_stats();
  let output_name = make_relative_from(&compiler.options.output.path, fixture_path);
  let rst = RstBuilder::default()
    .fixture(PathBuf::from(fixture_path))
    .actual(output_name)
    .build()
    .expect("TODO:");
  let warnings = stats.get_warnings();
  let errors = stats.get_errors();
  if !warnings.is_empty() && errors.is_empty() {
    println!(
      "Warning to compile in fixtrue {:?}, warnings: {:?}",
      fixture_path,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    )
  }
  if !errors.is_empty() {
    panic!(
      "Failed to compile in fixtrue {:?}, errors: {:?}",
      fixture_path,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    );
  }
  rst.assert();

  compiler
}
