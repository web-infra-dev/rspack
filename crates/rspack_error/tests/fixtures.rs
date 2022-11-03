use insta::Settings;
use rspack_binding_options::RawOptions;
use rspack_core::{CompilerOptions, Stats};
use rspack_test::{fixture, test_options::RawOptionsExt};

use std::path::PathBuf;

#[tokio::main]
pub async fn test_fixture<F: FnOnce(&Stats, Settings) -> rspack_error::Result<()>>(
  fixture_path: &PathBuf,
  f: F,
) -> rspack_error::Result<()> {
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let mut compiler = rspack::rspack(options, Default::default());

  let stats = compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));
  let mut settings = Settings::clone_current();
  settings.remove_snapshot_suffix();
  settings.set_snapshot_path(fixture_path);
  f(&stats, settings)
}

#[fixture("tests/fixtures/*")]
fn custom(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |stats, settings| {
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
        stats.emit_error_and_warning_string(false).unwrap(),
        dirname.as_str()
      );
    });
    Ok(())
  })
  .unwrap();
}

/// In concurrent scenario the file resolve order can't be guaranteed.
/// So we need to sort the diagnostic before writing into snapshot
#[fixture("tests/out_of_order/*")]
fn out_of_order(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |stats, settings| {
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
        stats.emit_error_and_warning_string(true).unwrap(),
        dirname.as_str()
      );
    });
    Ok(())
  })
  .unwrap();
}
