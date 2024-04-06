//! This is a complicated method for debugging the Rust core without running node.js
//! 1. Emit the config that the Rust side can consume:
//!   `cargo run --example build-cli -- `pwd`/examples/xxx/test.config.js --emit > test.config.json`
//! 2. Inside test.config.json, remove the loaders `module.rules` array. Specifying the loaders does not work yet.
//! 3. Run
//!     `cargo run --example build-cli -- test.config.json`
//! 4. The compiler will emit some errors on not being able to parse some of the files, e.g. .svg files.
//!    But if should probably succeed with "Build success".

use std::{env, path::PathBuf};

use rspack_binding_options::RawOptions;
use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::evaluate_to_json;
use rspack_tracing::enable_tracing_by_env;

#[tokio::main]
async fn main() {
  enable_tracing_by_env(&std::env::var("TRACE").ok().unwrap_or_default(), "stdout");

  let config = env::args().nth(1).expect("path");
  let emit = matches!(env::args().nth(2), Some(arg) if arg == "--emit");
  let config = PathBuf::from(config);
  let config = if config.is_absolute() {
    config
  } else {
    let cwd = env::current_dir().expect("current_dir");
    cwd.join(config).canonicalize().expect("canonicalize")
  };

  let raw = evaluate_to_json(&config);
  if emit {
    println!("{}", String::from_utf8_lossy(&raw));
    return;
  }
  let raw: RawOptions = serde_json::from_slice(&raw).expect("ok");
  let options = raw.try_into().expect("should be ok");
  let mut compiler = Compiler::new(options, Vec::new(), AsyncNativeFileSystem);
  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixture {config:?}, {e:#?}"));
  let stats = compiler.compilation.get_stats();
  let errors = stats.get_errors();
  if !errors.is_empty() {
    eprintln!(
      "Failed to compile in fixture {:?}, errors: {:?}",
      config,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    );
  }

  println!("Build success");
}
