use std::{
  env, fs,
  path::{Path, PathBuf},
};

use rspack_core::{Loader, LoaderRunner, ResourceData};
use rspack_loader_sass::{SassLoader, SassLoaderOptions};
use rspack_test::{fixture, test_fixture};
use sass_embedded::Url;

// UPDATE_SASS_LOADER_TEST=1 cargo test --package rspack_loader_sass test_fn_name -- --exact --nocapture
async fn loader_test(actual: impl AsRef<Path>, expected: impl AsRef<Path>) {
  let tests_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"))).join("tests");
  let expected_path = tests_path.join(expected);
  let actual_path = tests_path.join(actual);

  let url = Url::from_file_path(&actual_path.to_string_lossy().to_string()).unwrap();
  let result = LoaderRunner::new(ResourceData {
    resource: actual_path.to_string_lossy().to_string(),
    resource_path: url.path().to_owned(),
    resource_query: url.query().map(|q| q.to_owned()),
    resource_fragment: url.fragment().map(|f| f.to_owned()),
  })
  .run([&SassLoader::new(None, SassLoaderOptions::default()) as &dyn Loader])
  .await
  .unwrap();
  let result = result.content.try_into_string().unwrap();

  if env::var("UPDATE_SASS_LOADER_TEST").is_ok() {
    fs::write(expected_path, result).unwrap();
  } else {
    let expected = fs::read_to_string(expected_path).unwrap();
    assert_eq!(result, expected);
  }
}

#[tokio::test]
async fn rspack_importer() {
  loader_test("scss/language.scss", "expected/rspack_importer.css").await;
}

#[fixture("tests/fixtures/*")]
fn sass(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
