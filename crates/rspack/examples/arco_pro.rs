use std::{collections::HashMap, path::Path, time::Duration};

use rspack::bundler::{BundleOptions, Bundler};
use rspack_core::{BundleMode, BundleReactOptions, Loader, ResolveOption};
use sugar_path::PathSugar;
use tracing::{instrument, Instrument};

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
  let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let root = dir
    .join("../../examples/acro-pro")
    .normalize()
    .to_string_lossy()
    .to_string();
  let example = dir
    .join("../../examples/arco-pro/src/index.tsx")
    .normalize()
    .to_string_lossy()
    .to_string();

  let mut bundler = Bundler::new(
    BundleOptions {
      root: root,
      entries: vec![example.to_string()],
      outdir: "./dist".to_string(),
      code_splitting: Some(Default::default()),
      mode: BundleMode::Dev,
      react: BundleReactOptions {
        refresh: false,
        ..Default::default()
      },
      loader: HashMap::from_iter([
        ("css".to_string(), Loader::Css),
        ("less".to_string(), Loader::Less),
        ("sass".to_string(), Loader::Sass),
        ("scss".to_string(), Loader::Sass),
        ("svg".to_string(), Loader::DataURI),
      ]),
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
    vec![Box::new(rspack_plugin_mock_buitins::MockBuitinsPlugin)],
  );
  let build_future = async {
    bundler.build(None).await;
    tokio::time::sleep(Duration::from_millis(3000)).await;
    bundler
      .rebuild(
        dir
          .join("../../examples/arco-pro/src/components/NavBar/index.tsx")
          .normalize()
          .to_string_lossy()
          .to_string(),
      )
      .await;
  };
  build_future.instrument(tracing::info_span!("build")).await;
  // println!("assets: {:#?}", bundler.ctx.assets.lock().unwrap());
  bundler.write_assets_to_disk();
  // guard.lock().unwrap().as_mut().unwrap().flush();
  guard.map(|g| g.flush());
}
