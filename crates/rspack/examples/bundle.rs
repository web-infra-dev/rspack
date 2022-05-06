use rspack::bundler::{BundleOptions, Bundler};

#[tokio::main]
async fn main() {
  let guard = rspack::utils::log::enable_tracing_by_env_with_chrome_layer();
  let mut bundler = Bundler::new(
    BundleOptions {
      // entries: vec![
      //   "./fixtures/basic/entry-a.js".to_owned(),
      //   "./fixtures/basic/entry-b.js".to_owned(),
      // ],
      // entries: vec!["../../examples/react/index.js".to_owned()],
      entries: vec!["../../packages/rspack/node_modules/lodash-es/lodash.js".to_owned()],
      outdir: "./dist".to_string(),
      code_splitting: false,
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
