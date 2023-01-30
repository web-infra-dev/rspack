use std::path::{Path, PathBuf};

use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use rspack::Compiler;
use rspack_core::CompilerOptions;
use rspack_tracing::enable_tracing_by_env;

use crate::TestConfig;

#[tokio::main]
pub async fn test_fixture_rst(fixture_path: &Path) -> Compiler {
  enable_tracing_by_env();
  //avoid interference from previous testing
  let dist_dir = fixture_path.join("dist");
  if dist_dir.exists() {
    std::fs::remove_dir_all(dist_dir.clone()).expect("Should remove dist dir");
  }
  let options: CompilerOptions = TestConfig::compiler_options_from_fixture(fixture_path);
  // println!("{options:#?}");
  // let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  // println!("{options:#?}");
  let output_path = options.output.path.clone();
  let mut compiler = rspack::rspack(options, Default::default());
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
