

use rspack::bundler::{BundleOptions, Bundler};

#[tokio::main]
async fn main() {
    let mut bundler = Bundler::new(
        BundleOptions {
            // entries: vec!["./crates/rspack/fixtures/basic/entry-b.js".to_owned()],
            entries: vec!["./packages/rspack/node_modules/lodash-es/lodash.js".to_owned()],
            outdir: Some("./dist".to_string()),
            ..Default::default()
        },
        vec![],
    );
    bundler.generate().await;
    // println!("assets: {:#?}", bundler.ctx.assets.lock().unwrap());
    bundler.write_assets_to_disk();
}
