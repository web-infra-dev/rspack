use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn enabled() -> bool {
  std::env::var("RSPACK_BENCH_ENABLE_THREEJS_10X")
    .map(|value| value == "1")
    .unwrap_or(false)
}

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "threejs-10x",
    entry: "./src/index.js",
    swc_loader: false,
  })
}
