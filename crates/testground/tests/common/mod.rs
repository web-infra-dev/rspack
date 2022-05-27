use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
use rspack::bundler::Bundler;
use rspack_core::{BundleOptions, Plugin, RuntimeOptions};

#[derive(Default)]
pub struct HandyOverrideBundleOptions {
  pub sourcemap: bool,
  pub runtime_options: RuntimeOptions,
}

pub async fn compile(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
  let mut bundler = Bundler::new(options, plugins);
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}

/// [HandyOverrideBundleOptions]is used for override some bundle option, which make snapshot test result more readable,
/// for example, sometimes, we don't need that runtime code when testing  production code
pub async fn compile_fixture(
  fixture_dir_name: &str,
  handy_options: Option<HandyOverrideBundleOptions>,
) -> Bundler {
  let mut options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");

  if let Some(handy_options) = handy_options {
    options.runtime = handy_options.runtime_options;
    options.source_map = handy_options.sourcemap;
  }

  let mut bundler = Bundler::new(options, Default::default());
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}

pub async fn compile_fixture_with_plugins(
  fixture_dir_name: &str,
  plugins: Vec<Box<dyn Plugin>>,
) -> Bundler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");
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

pub mod prelude {
  pub use super::RawOptionsTestExt;

  pub use rspack_core::Plugin;
}
