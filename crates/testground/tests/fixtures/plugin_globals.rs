use rspack_core::RuntimeOptions;

use crate::common::{compile_fixture, HandyOverrideBundleOptions};

#[tokio::test]
async fn plugin_globals() {
  let bundler = compile_fixture(
    "plugin-globals",
    Some(HandyOverrideBundleOptions {
      runtime_options: RuntimeOptions {
        hmr: false,
        polyfill: false,
        module: false,
      },
      ..Default::default()
    }),
  )
  .await;
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
