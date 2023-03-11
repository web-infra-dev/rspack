mod termcolorful;
use std::{path::PathBuf, time::Instant};

use mimalloc_rust::GlobalMiMalloc;
use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::apply_from_fixture;
#[cfg(feature = "tracing")]
use rspack_tracing::enable_tracing_by_env_with_chrome_layer;
use termcolorful::println_string_with_fg_color;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static GLOBAL: GlobalMiMalloc = GlobalMiMalloc;

#[tokio::main]
async fn main() {
  let path_list = vec![
    // "examples/cjs-tree-shaking-basic",
    // "examples/basic",
    "examples/basic",
    // "examples/export-star-chain",
    // "examples/bbb",
    /* "examples/named-export-decl-with-src-eval",
     * "examples/side-effects-prune",
     * "examples/side-effects-two", */
  ];
  for p in path_list {
    println_string_with_fg_color(p, termcolorful::Color::Red);
    run(p).await;
  }
}

async fn run(relative_path: &str) {
  #[cfg(feature = "tracing")]
  let guard = enable_tracing_by_env_with_chrome_layer();
  let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
  let bundle_dir: PathBuf = manifest_dir.join(relative_path);
  println!("{bundle_dir:?}");
  let (options, plugins) = apply_from_fixture(&bundle_dir);
  #[cfg(feature = "hmr")]
  let options = {
    let mut options = options;
    use rspack_core::{CacheOptions, MemoryCacheOptions};
    // options.devtool = Default::default();
    options.builtins.minify_options = None;
    options.cache = CacheOptions::Memory(MemoryCacheOptions { max_generations: 0 });
    options.snapshot.resolve.timestamp = true;
    options.snapshot.module.timestamp = true;
    options
  };

  let start = Instant::now();
  // println!("{:?}", options);
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);

  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{e:?}, failed to compile in fixtrue {bundle_dir:?}"));
  compiler.compilation.get_stats().emit_diagnostics().unwrap();
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
