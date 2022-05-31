use crate::common::{compile_fixture, run_js_asset_in_node};

#[tokio::test]
async fn resolve_extensions_order() {
  let bundler = compile_fixture("resolve-extensions-order").await;
  let assets = bundler.bundle.context.assets.lock().unwrap();
  let js_asset = assets.get(0).unwrap();

  run_js_asset_in_node(js_asset);
}
