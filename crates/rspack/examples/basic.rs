use std::{collections::HashMap, path::Path};

use rspack_core::{log::enable_tracing_by_env, Compiler, CompilerOptions};
use sugar_path::PathSugar;

#[tokio::main]
async fn main() {
  enable_tracing_by_env();
  let mut compiler = Compiler::new(
    CompilerOptions {
      entries: HashMap::from([("main".to_string(), "./src/index.js".to_string().into())]),
      root: Path::new("./examples/react")
        .resolve()
        .to_string_lossy()
        .to_string(),
    },
    vec![
      Box::new(rspack_plugin_javascript::JsPlugin {}),
      Box::new(rspack_plugin_css::CssPlugin {}),
    ],
  );

  compiler.run().await.unwrap();
}
