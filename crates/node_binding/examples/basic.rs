use std::{collections::HashMap, path::Path};

use rspack::rspack;
use rspack_core::log;
use rspack_node::{normalize_bundle_options, RawOptions};

#[tokio::main]
async fn main() {
  let guard = log::enable_tracing_by_env_with_chrome_layer();
  let mut compiler = rspack(
    normalize_bundle_options(RawOptions {
      entry: HashMap::from([("main".to_string(), "./src/index.js".to_string())]),
      context: Some(
        Path::new("./examples/react")
          // .resolve()
          .to_string_lossy()
          .to_string(),
      ),
      ..Default::default()
    })
    .unwrap(),
    vec![],
  );

  compiler.compile().await.unwrap();

  println!(
    "entrypoints {:#?}",
    compiler
      .compilation
      .entrypoints
      .values()
      .next()
      .unwrap()
      .get_files(&compiler.compilation.chunk_graph)
  );

  if let Some(g) = guard {
    g.flush()
  }
}
