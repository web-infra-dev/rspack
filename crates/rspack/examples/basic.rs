use std::{collections::HashMap, path::Path};

use rspack::rspack;
use rspack_core::{log, CompilerOptions};
use sugar_path::PathSugar;

#[tokio::main]
async fn main() {
  let guard = log::enable_tracing_by_env_with_chrome_layer();
  let mut compiler = rspack(
    CompilerOptions {
      entries: HashMap::from([("main".to_string(), "./src/index.js".to_string().into())]),
      root: Path::new("./examples/react")
        .resolve()
        .to_string_lossy()
        .to_string(),
      ..Default::default()
    },
    vec![],
  );

  compiler.run().await.unwrap();

  if let Some(g) = guard {
    g.flush()
  }
}
