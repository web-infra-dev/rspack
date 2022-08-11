use rspack::rspack;
use rspack_core::{log, ParserOptions};
use rspack_node::{
  normalize_bundle_options, RawAssetParserDataUrlOption, RawAssetParserOptions, RawModuleOptions,
  RawOptions, RawOutputOptions, RawParserOptions,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
  let guard = log::enable_tracing_by_env_with_chrome_layer();
  let mut compiler = rspack(
    normalize_bundle_options(RawOptions {
      entry: HashMap::from([("main".to_string(), "./src/index.js".to_string())]),
      context: std::env::current_dir()
        .map(|dir| {
          dir
            .join("examples/react")
            // .resolve()
            .to_string_lossy()
            .to_string()
        })
        .ok(),
      output: Some(RawOutputOptions {
        public_path: Some(String::from("http://localhost:3000/")),
        ..RawOutputOptions::default()
      }),
      module: Some(RawModuleOptions {
        rules: vec![],
        parser: Some(RawParserOptions {
          asset: Some(RawAssetParserOptions {
            data_url_condition: Some(RawAssetParserDataUrlOption { max_size: Some(1) }),
          }),
        }),
      }),
      plugins: Some(json!(["html"])),
      ..Default::default()
    })
    .expect("Failed to normalized options"),
    vec![],
  );

  compiler.compile().await.unwrap();

  if let Some(g) = guard {
    g.flush()
  }
}
