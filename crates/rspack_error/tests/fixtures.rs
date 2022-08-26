use insta::Settings;
use rspack_binding_options::RawOptions;
use rspack_core::CompilerOptions;
use rspack_test::{fixture, test_options::RawOptionsExt};

use std::path::PathBuf;

#[tokio::main]
pub async fn test_fixture(fixture_path: &PathBuf) -> rspack_error::Result<()> {
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let mut compiler = rspack::rspack(options, Default::default());

  let stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));
  let mut settings = Settings::clone_current();
  settings.remove_snapshot_suffix();
  settings.set_snapshot_path(fixture_path);
  let dirname = fixture_path
    .components()
    .last()
    .unwrap()
    .as_os_str()
    .to_str()
    .unwrap()
    .to_owned();
  settings.bind(|| {
    insta::assert_snapshot!(
      dirname.as_str(),
      stats.emit_error_string().unwrap(),
      dirname.as_str()
    );
  });
  Ok(())
}

#[fixture("tests/fixtures/*")]
fn custom(fixture_path: PathBuf) {
  test_fixture(&fixture_path).unwrap();
}
