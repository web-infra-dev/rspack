use std::{collections::HashMap, path::Path};

use rspack::bundler::{BundleOptions, Bundler};
use rspack_core::{BundleMode, BundleReactOptions, Loader, ResolveOption};
use sugar_path::PathSugar;
use tracing::instrument;

#[instrument]
#[tokio::main]
async fn main() {
  // println!(
  //   "{:#?}",
  //   Path::new("./examples/arco-pro/src/")
  //     .resolve()
  //     .to_string_lossy()
  //     .to_string()
  //     + "/"
  // );
  let guard = rspack::utils::log::enable_tracing_by_env_with_chrome_layer();
  let mut bundler = Bundler::new(
    BundleOptions {
      entries: vec!["./examples/arco-pro/src/index.tsx".to_string()],
      outdir: "./dist".to_string(),
      code_splitting: true,
      mode: BundleMode::Dev,
      react: BundleReactOptions {
        refresh: true,
        ..Default::default()
      },
      loader: Some(HashMap::from_iter([
        ("json".to_string(), Loader::Json),
        ("less".to_string(), Loader::Text),
        ("svg".to_string(), Loader::DataURI),
      ])),
      resolve: ResolveOption {
        alias: vec![(
          "@/".to_string(),
          Some(
            Path::new("./examples/arco-pro/src/")
              .resolve()
              .to_string_lossy()
              .to_string()
              + "/",
          ),
        )],
        ..Default::default()
      },
      source_map: false,
      ..Default::default()
    },
    vec![],
  );
  bundler.build().await;
  // println!("assets: {:#?}", bundler.ctx.assets.lock().unwrap());
  bundler.write_assets_to_disk();
  // guard.lock().unwrap().as_mut().unwrap().flush();
  guard.map(|g| g.flush());
}
