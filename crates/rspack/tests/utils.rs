use rspack::bundler::{BundleOptions, Bundler};

use rspack_core::EntryItem;
use rspack_core::Plugin;

use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn compile(fixture_path: &str, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
  INIT.call_once(|| {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
      default_panic(info);
      std::process::exit(1);
    }));
  });
  compile_with_options(fixture_path, Default::default(), plugins)
}

pub fn compile_with_options(
  fixture_path: &str,
  options: BundleOptions,
  plugins: Vec<Box<dyn Plugin>>,
) -> Bundler {
  compile_with_options_inner(fixture_path, options, plugins)
}

#[tokio::main]
async fn compile_with_options_inner(
  fixture_path: &str,
  options: BundleOptions,
  plugins: Vec<Box<dyn Plugin>>,
) -> Bundler {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let fixtures_dir = dir.join("fixtures").join(fixture_path);
  let pkg_path = fixtures_dir.join("package.json");
  let pkg_content = fs::read_to_string(pkg_path);
  let mut pkg: Value = Value::default();
  if pkg_content.is_ok() {
    pkg = serde_json::from_str(&pkg_content.unwrap()).unwrap();
  }
  // use pkg.rspack.entry if set otherwise use index.js as entry
  let pkg_entry = pkg["rspack"].clone()["entry"].clone();
  let entry: HashMap<String, EntryItem> = {
    if pkg_entry.is_object() {
      let obj: HashMap<String, String> = serde_json::from_value(pkg_entry).unwrap();
      obj
        .into_iter()
        .map(|(id, value)| {
          let resolve_path = fixtures_dir.join(value).display().to_string();
          (id, resolve_path.into())
        })
        .collect()
    } else {
      let default_entry = fixtures_dir.join("index.js").to_str().unwrap().to_string();
      HashMap::from([("main".to_string(), default_entry.into())])
    }
  };
  let svgr = pkg["rspack"].clone()["svgr"].as_bool().unwrap_or(false);
  let dist = fixtures_dir.join("dist");
  println!("entry: {:?}", entry);
  println!("options: \n {:?}", options);
  let mut bundler = Bundler::new(
    BundleOptions {
      entries: entry.into_iter().map(From::from).collect(),
      outdir: dist.to_str().unwrap().to_string(),
      svgr,
      ..options
    },
    plugins,
  );
  bundler.build(None).await;
  bundler.write_assets_to_disk();
  bundler
}

pub fn assert_inline_sourcemap_in_pos(
  dist_code: &String,
  line_in_dist: u32,
  column_in_dist: u32,
  expected_in_source: &str,
) {
  const DATA_PREAMBLE: &str = "data:application/json;charset=utf-8;base64,";
  // TODO: should find last DATA_PREAMBLE.
  let index = dist_code.find(DATA_PREAMBLE).unwrap();
  let data_b64 = &dist_code[index + DATA_PREAMBLE.len()..];
  let data = base64::decode(data_b64).unwrap();
  let decoded_map = sourcemap::decode_slice(&data).unwrap();
  let token = decoded_map
    .lookup_token(line_in_dist, column_in_dist)
    .unwrap();
  let source_view = token.get_source_view().unwrap();
  let actual = source_view
    .get_line_slice(
      token.get_src_line(),
      token.get_src_col(),
      expected_in_source.len() as u32,
    )
    .unwrap();
  assert_eq!(actual, expected_in_source);
}
