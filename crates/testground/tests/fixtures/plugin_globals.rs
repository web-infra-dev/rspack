use crate::common::compile_fixture;

#[tokio::test]
async fn plugin_globals() {
  let bundler = compile_fixture("plugin-globals").await;
  insta::assert_snapshot!(
    &bundler
      .bundle
      .context
      .assets
      .lock()
      .unwrap()
      .get(0)
      .expect("failed to generate bundle")
      .source
  );
}
