use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
use rspack_core::{AssetContent, Compiler};

#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) -> Compiler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_path))
    .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {:?}", fixture_path));

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

  stats.assets().iter().for_each(|asset| {
    expected_files
      .keys()
      .cloned()
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|filename| {
        if asset.filename().ends_with(&filename) {
          if let AssetContent::String(content) = &asset.content() {
            let expected = String::from_utf8(expected_files.remove(&filename).unwrap())
              .expect("failed to convert file to utf8");
            similar_asserts::assert_str_eq!(
              content.trim(),
              expected.trim(),
              "Test failed in fixture:{:?}, the filename is {:?}",
              fixture_path,
              filename
            )
          } else if let AssetContent::Buffer(buf) = &asset.content() {
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

pub trait RawOptionsTestExt {
  fn from_fixture(fixture_path: &Path) -> Self;
}

impl RawOptionsTestExt for RawOptions {
  fn from_fixture(fixture_path: &Path) -> Self {
    let pkg_path = fixture_path.join("rspack.config.json");
    let mut options = {
      if pkg_path.exists() {
        let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
        let options: RawOptions = serde_json::from_str(&pkg_content).unwrap();
        options
      } else {
        RawOptions {
          entry: HashMap::from([(
            "main".to_string(),
            fixture_path.join("index.js").to_str().unwrap().to_string(),
          )]),
          ..Default::default()
        }
      }
    };
    assert!(
      options.context.is_none(),
      "You should not specify `root` in config. It would probably resolve to a wrong path"
    );
    options.context = Some(fixture_path.to_str().unwrap().to_string());
    options
  }
}
