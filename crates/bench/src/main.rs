mod termcolorful;
use std::str::FromStr;
use std::{path::PathBuf, time::Instant};

use rspack_core::{BoxPlugin, Compiler, CompilerOptions, TreeShaking, UsedExportsOption};
use rspack_fs::AsyncNativeFileSystem;
use rspack_plugin_javascript::{FlagDependencyExportsPlugin, FlagDependencyUsagePlugin};
use rspack_testing::apply_from_fixture;
#[cfg(feature = "tracing")]
use rspack_tracing::{enable_tracing_by_env, enable_tracing_by_env_with_chrome_layer};
use termcolorful::println_string_with_fg_color;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(all(
  target_os = "linux",
  target_env = "gnu",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Default, Clone, Copy)]
enum Layer {
  #[default]
  Logger,
  Chrome,
}

impl FromStr for Layer {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "chrome" => Self::Chrome,
      "logger" => Self::Logger,
      _ => unimplemented!("Unknown layer {s}, please use one of `chrome`, `logger` "),
    })
  }
}

#[tokio::main]
async fn main() {
  #[cfg(feature = "tracing")]
  let layer = std::env::var("layer")
    .ok()
    .and_then(|var| Layer::from_str(&var).ok())
    .unwrap_or_default();
  let path_list = vec![
    // "examples/cjs-tree-shaking-basic",
    // "examples/basic",
    "crates/rspack/tests/tree-shaking/export_star",
    // "examples/export-star-chain",
    // "examples/bbb",
    /* "examples/named-export-decl-with-src-eval",
     * "examples/side-effects-prune",
     * "examples/side-effects-two", */
  ];
  for p in path_list {
    println_string_with_fg_color(p, termcolorful::Color::Red);
    run(
      p,
      #[cfg(feature = "tracing")]
      layer,
    )
    .await;
  }
}

async fn run(relative_path: &str, #[cfg(feature = "tracing")] layer: Layer) {
  #[cfg(feature = "tracing")]
  let guard = match layer {
    Layer::Logger => enable_tracing_by_env(),
    Layer::Chrome => enable_tracing_by_env_with_chrome_layer(),
  };
  let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
  let bundle_dir: PathBuf = manifest_dir.join(relative_path);
  println!("{bundle_dir:?}");
  let (mut options, mut plugins): (CompilerOptions, Vec<BoxPlugin>) =
    apply_from_fixture(&bundle_dir);

  options.experiments.rspack_future.new_treeshaking = true;
  options.optimization.provided_exports = true;
  options.optimization.inner_graph = true;
  options.optimization.used_exports = UsedExportsOption::True;
  options.builtins.tree_shaking = TreeShaking::False;

  // if options.optimization.side_effects.is_enable() {
  //   plugins.push(Box::<SideEffectsFlagPlugin>::default());
  // }
  plugins.push(Box::<FlagDependencyExportsPlugin>::default());
  plugins.push(Box::<FlagDependencyUsagePlugin>::default());
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
    .unwrap_or_else(|e| panic!("{e:?}, failed to compile in fixture {bundle_dir:?}"));
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
