use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
use rspack::bundler::Bundler;
use rspack_core::{BundleOptions, Plugin};
use sugar_path::PathSugar;

pub async fn compile(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
  let mut bundler = Bundler::new(options, plugins);
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}

pub async fn compile_fixture(fixture_dir_name: &str) -> Bundler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name));
  let mut bundler = Bundler::new(options, Default::default());
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}

pub async fn compile_fixture_with_plugins(
  fixture_dir_name: &str,
  plugins: Vec<Box<dyn Plugin>>,
) -> Bundler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name));
  let mut bundler = Bundler::new(options, plugins);
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}
pub trait RawOptionsTestExt {
  fn from_fixture(fixture: &str) -> Self;
}

impl RawOptionsTestExt for RawOptions {
  fn from_fixture(fixture_path: &str) -> Self {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = dir.join("fixtures").join(fixture_path);
    let pkg_path = fixtures_dir.join("rspack.config.json");
    let options = {
      if pkg_path.exists() {
        let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
        let mut options: RawOptions = serde_json::from_str(&pkg_content).unwrap();
        options.entries = options
          .entries
          .into_iter()
          .map(|(name, src)| {
            (
              name,
              fixtures_dir
                // FIXME: We sould not manually do it.
                .join(Path::new(&src))
                .resolve()
                .to_str()
                .unwrap()
                .to_string(),
            )
          })
          .collect();
        options
      } else {
        RawOptions {
          entries: HashMap::from([(
            "main".to_string(),
            fixtures_dir.join("index.js").to_str().unwrap().to_string(),
          )]),
          root: Some(fixtures_dir.to_str().unwrap().to_string()),
          ..Default::default()
        }
      }
    };
    options
  }
}

pub mod preclude {
  pub use super::RawOptionsTestExt;
  pub use rspack_core::Plugin;
}
