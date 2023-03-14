use std::path::PathBuf;

use insta::Settings;
use rspack_core::{Compiler, Stats};
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::{apply_from_fixture, fixture};
use rspack_tracing::enable_tracing_by_env;

#[tokio::main]
pub async fn test_fixture<F: FnOnce(&Stats, Settings) -> rspack_error::Result<()>>(
  fixture_path: &PathBuf,
  f: F,
) -> rspack_error::Result<()> {
  let (options, plugins) = apply_from_fixture(fixture_path);
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);

  compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {fixture_path:?}"));
  let stats = compiler.compilation.get_stats();
  let mut settings = Settings::clone_current();
  settings.remove_snapshot_suffix();
  settings.set_snapshot_path(fixture_path);
  f(&stats, settings)
}

#[fixture("tests/fixtures/*", exclude("export_star_error"))]
fn custom(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path, |stats, settings| {
    let dirname = fixture_path
      .components()
      .last()
      .expect("TODO:")
      .as_os_str()
      .to_str()
      .expect("TODO:")
      .to_owned();
    settings.bind(|| {
      insta::assert_snapshot!(
        dirname.as_str(),
        stats.emit_diagnostics_string(false).expect("TODO:"),
        dirname.as_str()
      );
    });
    Ok(())
  })
  .expect("TODO:");
}

/// In concurrent scenario the file resolve order can't be guaranteed.
/// So we need to sort the diagnostic before writing into snapshot
#[fixture("tests/out_of_order/*")]
fn out_of_order(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |stats, settings| {
    let dirname = fixture_path
      .components()
      .last()
      .expect("TODO:")
      .as_os_str()
      .to_str()
      .expect("TODO:")
      .to_owned();
    settings.bind(|| {
      insta::assert_snapshot!(
        dirname.as_str(),
        stats.emit_diagnostics_string(true).expect("TODO:"),
        dirname.as_str()
      );
    });
    Ok(())
  })
  .expect("TODO:");
}
