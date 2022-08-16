use rspack::rspack;
use rspack_core::log;
use rspack_node::{normalize_bundle_options, RawOptions};

#[tokio::main]
async fn main() {
  let guard = log::enable_tracing_by_env_with_chrome_layer();

  let context = std::env::current_dir().unwrap().join("examples/react/");
  let config_path = context
    .join("test.config.json")
    .to_string_lossy()
    .to_string();
  let config = std::fs::read_to_string(config_path).unwrap();
  let options: RawOptions = serde_json::from_str(&config).unwrap();
  let mut compiler = rspack(
    normalize_bundle_options(RawOptions {
      context: Some(context.to_string_lossy().to_string()),
      ..options
    })
    .unwrap(),
    vec![],
  );

  compiler.compile().await.unwrap();

  if let Some(g) = guard {
    g.flush()
  }
}
