use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture_css, test_fixture_css_modules};

#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture_css(&fixture_path);
}

#[fixture("tests/fixtures/custom/*", exclude("tests/fixtures/custom/modules-*"))]
fn custom(fixture_path: PathBuf) {
  test_fixture_css(&fixture_path);
}

#[fixture("tests/fixtures/custom/modules-*")]
fn custom_modules(fixture_path: PathBuf) {
  test_fixture_css_modules(&fixture_path);
}
