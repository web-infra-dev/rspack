use std::path::PathBuf;

use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::apply_from_fixture;

#[tokio::main]
async fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  let config_path = PathBuf::from(
    &args
      .get(1)
      .unwrap_or(&String::from("crates/rspack_build/fixture")),
  );
  run(config_path).await;
}

async fn run(config_path: PathBuf) {
  let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  let abs_path = if config_path.is_absolute() {
    config_path
  } else {
    manifest_dir.join(config_path)
  };
  let config_path = abs_path.join("test.config.json");
  if !config_path.exists() {
    panic!("{config_path:?} not exits, please make sure {config_path:?} exit")
  }
  let (options, plugins) = apply_from_fixture(&abs_path);
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);
  compiler
    .build()
    .await
    .unwrap_or_else(|err| panic!("{err:?}, failed to compile {abs_path:?}"));
}
