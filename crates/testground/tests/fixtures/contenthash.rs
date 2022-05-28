use crate::common::compile_fixture;

// #[tokio::test]
async fn content_hash() {
  let bundler = compile_fixture("contenthash").await;
  let assets = bundler.bundle.context.assets.lock().unwrap();
  dbg!(&assets);
  let assets_filename_list = assets
    .iter()
    .map(|asset| asset.filename.clone())
    .collect::<Vec<_>>();

  assert!(assets_filename_list.contains(&"main.js".to_string()));
  assert!(assets_filename_list.contains(&"83c71f5ed4562024567042298cf56afe.js".to_string()));
}
