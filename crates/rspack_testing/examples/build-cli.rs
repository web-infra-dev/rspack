//! cargo run --example build-cli -- `pwd`/examples/xxx/test.config.js --emit

use std::{env, path::PathBuf};

use rspack_binding_options::{RawOptions, RawOptionsApply};
use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::evaluate_to_json;
use rspack_tracing::enable_tracing_by_env;

#[tokio::main]
async fn main() {
  enable_tracing_by_env();

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
  }
  let raw: RawOptions = serde_json::from_slice(&raw).expect("ok");
  let mut plugins = Vec::new();
  let options = raw.apply(&mut plugins).expect("should be ok");
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);
  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixtrue {config:?}, {e:#?}"));
  let stats = compiler.compilation.get_stats();
  let errors = stats.get_errors();
  if !errors.is_empty() {
    eprintln!(
      "Failed to compile in fixtrue {:?}, errors: {:?}",
      config,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    );
  }

  println!("Build success");
}
