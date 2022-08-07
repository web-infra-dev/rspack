use std::collections::HashMap;

use rspack::rspack;
use rspack_core::log;
use rspack_node::{normalize_bundle_options, RawOptions, RawOutputOptions};

#[tokio::main]
async fn main() {
  let guard = log::enable_tracing_by_env_with_chrome_layer();
  let mut compiler = rspack(
    normalize_bundle_options(RawOptions {
      entry: HashMap::from([("main".to_string(), "./src/index.js".to_string())]),
      context: Some(
        std::env::current_dir()
          .unwrap()
          .join("examples/react")
          // .resolve()
          .to_string_lossy()
          .to_string(),
      ),
      output: Some(RawOutputOptions {
        public_path: Some(String::from("http://localhost:3000/")),
        ..RawOutputOptions::default()
      }),
      ..Default::default()
    })
    .unwrap(),
    vec![],
  );

  compiler.compile().await.unwrap();

  if let Some(g) = guard {
    g.flush()
  }
}
