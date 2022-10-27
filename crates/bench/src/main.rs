#[cfg(feature = "tracing")]
use rspack_tracing::enable_tracing_by_env_with_chrome_layer;
use std::{path::PathBuf, time::Instant};

use mimalloc_rust::GlobalMiMalloc;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static GLOBAL: GlobalMiMalloc = GlobalMiMalloc;
use rspack_test::read_test_config_and_normalize;
#[tokio::main]
async fn main() {
  #[cfg(feature = "tracing")]
  let guard = enable_tracing_by_env_with_chrome_layer();
  let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
  let bundle_dir: PathBuf = manifest_dir.join("crates/rspack/tests/tree-shaking/export_star");
  println!("{:?}", bundle_dir);
  let mut options = read_test_config_and_normalize(&bundle_dir);
  options.__emit_error = true;

  let start = Instant::now();
  // println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
  println!("{:?}", start.elapsed());
  #[cfg(feature = "tracing")]
  {
    if let Some(guard) = guard {
      guard.flush();
    }
  }
}
