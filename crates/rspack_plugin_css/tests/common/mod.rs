use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
// use rspack::Compiler;
use rspack_core::{Asset, Compiler, CompilerOptions, Plugin};

pub async fn compile(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  let mut bundler = Compiler::new(options, plugins);
  bundler.run().await;
  bundler
}

pub async fn compile_fixture(fixture_dir_name: &str) -> Compiler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");
  let mut bundler = Compiler::new(options, Default::default());
  bundler.run().await.expect("failed to build");
  bundler
}

pub async fn test_fixture_css(fixture_dir_name: &str) -> Compiler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");
  let mut bundler = Compiler::new(options, Default::default());

  let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let expected_dir_path = dir.join("fixtures").join(fixture_dir_name).join("expected");
  let mut expected = std::fs::read_dir(expected_dir_path)
    .unwrap()
    .flat_map(|entry| entry.ok())
    .filter_map(|entry| {
      let content = std::fs::read_to_string(entry.path()).ok()?;

      Some((entry.file_name().to_string_lossy().to_string(), content))
    })
    .collect::<HashMap<_, _>>();
  let assets = bundler.run().await.unwrap();
  // assets.iter().for_each(|asset| {
  //   expected
  //     .keys()
  //     .cloned()
  //     .collect::<Vec<_>>()
  //     .into_iter()
  //     .for_each(|filename| {
  //       if asset.filename.ends_with(&filename) {
  //         assert_eq!(
  //           asset.source.trim(),
  //           expected.remove(&filename).unwrap().trim(),
  //           "filename {:?}",
  //           filename
  //         )
  //       };
  //     });
  // });
  assert!(expected.is_empty());
  bundler
}

// pub async fn compile_fixture_with_plugins(
//   fixture_dir_name: &str,
//   plugins: Vec<Box<dyn Plugin>>,
// ) -> Compiler {
//   let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
//     .expect("failed to normalize");
//   let mut bundler = Compiler::new(options, plugins);
//   bundler.build(None).await.expect("failed to build");
//   bundler.write_assets_to_disk();
//   bundler
// }
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
