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
  let bundle_dir: PathBuf = manifest_dir.join("benchcases/three");
  println!("{:?}", bundle_dir);
  let mut options = read_test_config_and_normalize(&bundle_dir);
  options.__emit_error = true;
  #[cfg(feature = "hmr")]
  {
    use rspack_core::{CacheOptions, MemoryCacheOptions, Minification};
    // options.devtool = Default::default();
    options.builtins.minify = Minification {
      enable: false,
      passes: 0,
    };
    options.cache = CacheOptions::Memory(MemoryCacheOptions { max_generations: 0 });
    options.snapshot.resolve_build_dependencies.timestamp = true;
    options.snapshot.build_dependencies.timestamp = true;
    options.snapshot.resolve.timestamp = true;
    options.snapshot.module.timestamp = true;
  }

  let start = Instant::now();
  // println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
  println!("{:?}", start.elapsed());
  #[cfg(feature = "hmr")]
  {
    let entry_js_path = bundle_dir.join("src/entry.js");
    let index_js_content = std::fs::read_to_string(&entry_js_path).expect("TODO:");
    // change file
    std::fs::write(&entry_js_path, index_js_content.clone() + "\n //").expect("TODO:");
    let start = Instant::now();
    compiler.build().await.expect("TODO:");
    println!("{:?}", start.elapsed());
    // remove a import
    std::fs::write(&entry_js_path, "//".to_string() + &index_js_content.clone()).expect("TODO:");
    let start = Instant::now();
    compiler.build().await.expect("TODO:");
    println!("{:?}", start.elapsed());
    // recovery file
    std::fs::write(&entry_js_path, index_js_content).expect("TODO:");
    let start = Instant::now();
    compiler.build().await.expect("TODO:");
    println!("{:?}", start.elapsed());
  }

  #[cfg(feature = "tracing")]
  {
    if let Some(guard) = guard {
      guard.flush();
    }
  }
}
