use std::{collections::HashMap, path::Path};

use node_binding::{normalize_bundle_options, RawOptions, RawOutputOptions};
use rspack::bundler::Bundler;
use rspack_core::{Asset, BundleOptions, Plugin};

pub async fn compile(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
  let mut bundler = Bundler::new(options, plugins);
  bundler.build(None).await;
  bundler
}

pub async fn compile_fixture(fixture_dir_name: &str) -> Bundler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");
  let mut bundler = Bundler::new(options, Default::default());
  bundler.build(None).await.expect("failed to build");
  bundler
}

pub async fn compile_fixture_with_plugins(
  fixture_dir_name: &str,
  plugins: Vec<Box<dyn Plugin>>,
) -> Bundler {
  let options = normalize_bundle_options(RawOptions::from_fixture(fixture_dir_name))
    .expect("failed to normalize");
  let mut bundler = Bundler::new(options, plugins);
  bundler.build(None).await.expect("failed to build");
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
    let outdir = fixtures_dir.join("dist").to_str().unwrap().to_string();
    let mut options = {
      if pkg_path.exists() {
        let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
        let mut options: RawOptions = serde_json::from_str(&pkg_content).unwrap();
        if options.output.is_none() {
          options.output = Some(RawOutputOptions {
            outdir: Some(outdir),
            ..Default::default()
          });
        }
        options
      } else {
        RawOptions {
          entries: HashMap::from([(
            "main".to_string(),
            fixtures_dir.join("index.js").to_str().unwrap().to_string(),
          )]),
          output: Some(RawOutputOptions {
            outdir: Some(outdir),
            ..Default::default()
          }),
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

pub fn run_js_asset_in_node(js_asset: &Asset) {
  let filename = &js_asset.filename;
  let source = &js_asset.source;
  // TODO: should optimized
  // WARNING: `node eval` do not had module context.
  let command_result = std::process::Command::new("node")
    .args(["-e", source])
    .output();
  deal_node_command_output(&command_result, filename);
}

pub fn run_js_output_in_node(js_asset: &Asset, options: &BundleOptions) {
  let filename = &js_asset.filename;
  let output_file = Path::new(&options.outdir)
    .join(filename)
    .display()
    .to_string();
  // TODO: should optimized
  let command_result = std::process::Command::new("node")
    .args([output_file])
    .output();
  deal_node_command_output(&command_result, filename);
}

fn deal_node_command_output(output: &std::io::Result<std::process::Output>, filename: &str) {
  match output {
    Ok(result) => {
      if !result.stderr.is_empty() {
        panic!(
          "run {filename} failed.\n Error message: {}",
          std::str::from_utf8(&result.stderr).unwrap()
        )
      }
    }
    Err(err) => panic!("run {filename} failed.\n Error message {err}"),
  }
}
