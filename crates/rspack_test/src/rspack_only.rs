use crate::test_options::RawOptionsExt;
use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use rspack_binding_options::RawOptions;
use rspack_core::CompilerOptions;
use rspack_core::Stats;
use rspack_tracing::enable_tracing_by_env;
use std::path::{Path, PathBuf};

use rspack::Compiler;

#[tokio::main]
pub async fn test_fixture<F>(fixture_path: &Path, custom_convert_options: F) -> Compiler
where
  F: Fn(CompilerOptions) -> CompilerOptions,
{
  enable_tracing_by_env();
  //avoid interference from previous testing
  let dist_dir = fixture_path.join("dist");
  if dist_dir.exists() {
    std::fs::remove_dir_all(dist_dir.clone()).unwrap();
  }
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let options = custom_convert_options(options);
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

  if !stats.to_description().unwrap().errors.is_empty() {
    panic!(
      "Failed to compile in fixtrue {:?}, errors: {:?}",
      fixture_path,
      stats.emit_diagnostics_string(true).unwrap()
    );
  }

  rst.assert();
  compiler
}

pub fn options_noop(options: CompilerOptions) -> CompilerOptions {
  options
}

pub fn add_entry_runtime(mut options: CompilerOptions) -> CompilerOptions {
  for (_, entry) in options.entry.iter_mut() {
    entry.runtime = Some("runtime".to_string());
  }
  options
}
