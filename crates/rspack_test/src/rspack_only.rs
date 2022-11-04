use crate::test_options::RawOptionsExt;
use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use rspack_binding_options::RawOptions;
use rspack_core::CompilerOptions;
use rspack_core::Stats;
use rspack_tracing::enable_tracing_by_env;
use std::path::{Path, PathBuf};

use rspack::Compiler;

#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) -> Compiler {
  enable_tracing_by_env();
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let output_path = options.output.path.clone();
  let mut compiler = rspack::rspack(options, Default::default());

  let stats: Stats = compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));
  let output_name = make_relative_from(Path::new(&output_path), fixture_path);
  let rst = RstBuilder::default()
    .fixture(PathBuf::from(fixture_path))
    .actual(output_name)
    .build()
    .unwrap();

  if !stats.to_description().errors.is_empty() {
    panic!(
      "Failed to compile in fixtrue {:?}, errors: {:?}",
      fixture_path,
      stats.emit_diagnostics_string(true).unwrap()
    );
  }

  rst.assert();
  compiler
}
