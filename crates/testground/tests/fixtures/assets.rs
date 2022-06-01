use crate::common::{compile_fixture, run_js_output_in_node};
use std::collections::HashMap;

#[tokio::test]
async fn asset_emitted() {
  let bundler = compile_fixture("asset-emitted").await;
  let assets = bundler.bundle.context.assets.lock().unwrap();
  let map: HashMap<String, rspack_core::Asset> = assets
    .iter()
    .map(|asset| {
      (
        asset.filename.to_string(),
        rspack_core::Asset {
          filename: asset.filename.to_string(),
          source: asset.source.to_string(),
        },
      )
    })
    .collect();

  assert_eq!(map.len(), 2);
  assert!(map.contains_key("main.js"));
  assert!(map.contains_key("module_js.js"));
  let main_asset = map.get("main.js").unwrap();
  run_js_output_in_node(main_asset, &bundler.options);
}
