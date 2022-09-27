#[cfg(feature = "tracing")]
use rspack_core::log::enable_tracing_by_env_with_chrome_layer;
use std::{path::PathBuf, time::Instant};

use mimalloc_rust::GlobalMiMalloc;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static GLOBAL: GlobalMiMalloc = GlobalMiMalloc;
use rspack_test::read_test_config_and_normalize;
fn main() {
  let worker_thread = std::env::var("WORKER_THREAD")
    .ok()
    .and_then(|item| item.parse::<usize>().ok())
    .unwrap_or(8);
  println!("worker_thread: {}", worker_thread);
  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(worker_thread)
    .build()
    .unwrap();
  rt.block_on(async {
    #[cfg(feature = "tracing")]
    let guard = enable_tracing_by_env_with_chrome_layer();
    let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
    // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
    let bundle_dir: PathBuf = manifest_dir.join("benchcases/three");
    println!("{:?}", bundle_dir);
    let mut options = read_test_config_and_normalize(&bundle_dir);

    options.emit_error = true;
    let start = Instant::now();
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
  });
}
