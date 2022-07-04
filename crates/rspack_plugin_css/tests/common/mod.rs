use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
// use rspack::Compiler;
use rspack_core::{AssetContent, Compiler};

// pub async fn compile(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Compiler {
//   let mut bundler = Compiler::new(options, plugins);
//   bundler.run().await.unwrap();
//   bundler
// }

// pub async fn compile_fixture(fixture_dir_name: &str) -> Compiler {
//   let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
//     .expect("failed to normalize");
//   let mut bundler = Compiler::new(options, Default::default());
//   bundler.run().await.expect("failed to build");
//   bundler
// }

pub async fn test_fixture_css(fixture_dir_name: &str) -> Compiler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {}", fixture_dir_name));

  let mut compiler = rspack::rspack(options, Default::default());

  let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let expected_dir_path = dir.join("fixtures").join(fixture_dir_name).join("expected");

  let mut expected = std::fs::read_dir(expected_dir_path)
    .unwrap()
    .flat_map(|entry| entry.ok())
    .filter_map(|entry| {
      let content = std::fs::read(entry.path()).ok()?;
      Some((entry.file_name().to_string_lossy().to_string(), content))
    })
    .collect::<HashMap<_, _>>();

  let stats = compiler.run().await.unwrap();

  stats.assets().iter().for_each(|asset| {
    expected
      .keys()
      .cloned()
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|filename| {
        if asset.filename().ends_with(&filename) {
          if let AssetContent::String(content) = &asset.content() {
            let expected = String::from_utf8(expected.remove(&filename).unwrap())
              .expect("failed to convert file to utf8");
            assert_eq!(
              content.trim(),
              expected.trim(),
              "CSS test failed in fixture:{}, the filename is {:?}",
              fixture_dir_name,
              filename
            )
          } else if let AssetContent::Buffer(buf) = &asset.content() {
            assert_eq!(
              buf,
              &expected.remove(&filename).unwrap(),
              "CSS test failed in fixture:{}, the filename is {:?}",
              fixture_dir_name,
              filename
            )
          }
        };
      });
  });
  assert!(
    expected.is_empty(),
    "files {:?} are not visited",
    expected.keys().collect::<Vec<_>>()
  );
  compiler
}

pub trait RawOptionsTestExt {
  fn from_fixture(fixture: &str) -> Self;
}

impl RawOptionsTestExt for RawOptions {
  fn from_fixture(fixture_path: &str) -> Self {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = dir.join("fixtures").join(fixture_path);
    let pkg_path = fixtures_dir.join("rspack.config.json");
    let mut options = {
      if pkg_path.exists() {
        let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
        let options: RawOptions = serde_json::from_str(&pkg_content).unwrap();
        options
      } else {
        RawOptions {
          entries: HashMap::from([(
            "main".to_string(),
            fixtures_dir.join("index.js").to_str().unwrap().to_string(),
          )]),
          ..Default::default()
        }
      }
    };
    assert!(
      options.root.is_none(),
      "You should not specify `root` in config. It would probably resolve to a wrong path"
    );
    options.root = Some(fixtures_dir.to_str().unwrap().to_string());
    options
  }
}

// pub mod prelude {
//   pub use super::RawOptionsTestExt;

//   pub use rspack_core::Plugin;
// }
