use std::path::{Path, PathBuf};

use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use rspack_core::{Compiler, CompilerOptions};
use rspack_tracing::enable_tracing_by_env;

use crate::apply_from_fixture;

pub fn test_fixture(fixture_path: &Path) -> Compiler {
  test_fixture_with_modify(fixture_path, |i| i)
}

#[tokio::main]
pub async fn test_fixture_with_modify(
  fixture_path: &Path,
  modify: impl Fn(CompilerOptions) -> CompilerOptions,
) -> Compiler {
  enable_tracing_by_env();
  //avoid interference from previous testing
  let dist_dir = fixture_path.join("dist");
  if dist_dir.exists() {
    std::fs::remove_dir_all(dist_dir.clone()).expect("Should remove dist dir");
  }
  let (options, plugins) = apply_from_fixture(fixture_path);
  let options = modify(options);
  let output_path = options.output.path.clone();
  let mut compiler = Compiler::new(options, plugins);
  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixtrue {fixture_path:?}, {e:#?}"));
  let stats = compiler.compilation.get_stats();
  let output_name = make_relative_from(Path::new(&output_path), fixture_path);
  let rst = RstBuilder::default()
    .fixture(PathBuf::from(fixture_path))
    .actual(output_name)
    .build()
    .expect("TODO:");

  let errors = stats.get_errors();
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
