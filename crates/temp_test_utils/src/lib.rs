use std::{collections::HashMap, path::Path};

use rspack_core::{AssetContent, Compiler, CompilerOptions};
mod test_options;
pub use test_options::TestOptions;

#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) -> Compiler {
  let options: CompilerOptions = TestOptions::from_fixture(fixture_path).into();

  let mut compiler = rspack::rspack(options, Default::default());

  let expected_dir_path = fixture_path.join("expected");

  let stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));

  let mut expected_files = std::fs::read_dir(expected_dir_path)
    .expect("failed to read `expected` dir")
    .flat_map(|entry| entry.ok())
    .filter_map(|entry| {
      let content = std::fs::read(entry.path()).ok()?;
      Some((entry.file_name().to_string_lossy().to_string(), content))
    })
    .collect::<HashMap<_, _>>();

  stats
    .__should_only_used_in_tests_assets()
    .iter()
    .for_each(|(asset_filename, asset)| {
      expected_files
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|filename| {
          if asset_filename.ends_with(&filename) {
            if let AssetContent::String(content) = &asset.source() {
              let expected = String::from_utf8(expected_files.remove(&filename).unwrap())
                .expect("failed to convert file to utf8");
              similar_asserts::assert_eq!(
                content.trim(),
                expected.trim(),
                "Test failed in fixture:{:?}, the filename is {:?}",
                fixture_path,
                filename
              )
            } else if let AssetContent::Buffer(buf) = &asset.source() {
              similar_asserts::assert_eq!(
                buf,
                &expected_files.remove(&filename).unwrap(),
                "Test failed in fixture:{:?}, the filename is {:?}",
                fixture_path,
                filename
              )
            }
          };
        });
    });
  assert!(
    expected_files.is_empty(),
    "files {:?} are not visited in {:?}",
    expected_files.keys().collect::<Vec<_>>(),
    fixture_path
  );
  compiler
}
