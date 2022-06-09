mod utils;

use std::collections::HashMap;

use rspack::bundler::BundleOptions;
use rspack_core::ResolveOption;
use utils::compile_with_options;

#[test]
fn alias() {
  let bundler = compile_with_options(
    "alias",
    BundleOptions {
      resolve: ResolveOption {
        alias: HashMap::from_iter([
          ("./wrong".to_string(), Some("./ok".to_string())),
          ("@/".to_string(), Some("./src/".to_string())),
        ]),
        ..Default::default()
      },
      ..Default::default()
    },
    vec![],
  );
  let assets = bundler.bundle.context.assets.lock().unwrap();
  let dist = assets.get(0).unwrap();
  let source = &dist.source;
  assert!(!source.contains("wrong.js"));
  assert!(!source.contains('@'));
  assert!(source.contains("ok.js"));
  assert!(source.contains("at.js"));
}
