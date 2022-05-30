use std::{collections::HashMap, path::Path};

use rspack::bundler::{BundleOptions, Bundler};
use rspack_core::{BundleMode, BundleReactOptions};
use tracing::instrument;

#[instrument]
#[tokio::main]
async fn main() {
  let guard = rspack::utils::log::enable_tracing_by_env_with_chrome_layer();
  let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let example = dir
    .join("../../examples/react/src/index.js")
    .to_string_lossy()
    .to_string();
  let mut bundler = Bundler::new(
    BundleOptions {
      // entries: vec![
      //   "./fixtures/basic/entry-a.js".to_owned(),
      //   "./fixtures/basic/entry-b.js".to_owned(),
      // ],
      entries: HashMap::from([("main".to_string(), example.into())]),
      // entries: vec!["../../packages/rspack/node_modules/lodash-es/lodash.js".to_owned()],
      outdir: "./dist".to_string(),
      mode: BundleMode::Dev,
      react: BundleReactOptions {
        refresh: true,
        ..Default::default()
      },
      ..Default::default()
    },
    vec![],
  );
  bundler.build(None).await;
  // println!("assets: {:#?}", bundler.ctx.assets.lock().unwrap());
  bundler.write_assets_to_disk();
  // guard.lock().unwrap().as_mut().unwrap().flush();
  if let Some(g) = guard {
    g.flush()
  }
}
