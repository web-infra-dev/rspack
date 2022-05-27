use rspack_core::RuntimeOptions;

use crate::common::{compile_fixture, HandyOverrideBundleOptions};

#[tokio::test]
async fn constant_folding() {
  let bundler = compile_fixture(
    "constant-folding",
    Some(HandyOverrideBundleOptions {
      runtime_options: RuntimeOptions {
        hmr: false,
        polyfill: false,
        module: false,
      },
    }),
  )
  .await;
  let code = bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .expect("failed to generate bundle")
    .source
    .to_owned();
  insta::assert_snapshot!(code);
}
