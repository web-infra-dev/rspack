use crate::{helper::make_relative_from, rst::RstBuilder};
use rspack_binding_options::RawOptions;
use rspack_core::CompilerOptions;
use std::path::{Path, PathBuf};
use temp_test_utils::test_options::RawOptionsExt;

use rspack::Compiler;

#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) -> Compiler {
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let output_path = options.output.path.clone();
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));
  let output_name = make_relative_from(Path::new(&output_path), fixture_path);
  let rst = RstBuilder::default()
    .fixture(PathBuf::from(fixture_path))
    .actual(output_name)
    .build()
    .unwrap();

  rst.assert();
  compiler
}
