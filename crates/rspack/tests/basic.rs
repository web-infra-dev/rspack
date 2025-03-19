use rspack::builder::{Builder as _, Devtool};
use rspack_core::Compiler;
use rspack_paths::Utf8Path;

#[tokio::test(flavor = "multi_thread")]
async fn basic() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/basic"))
    .entry("main", "./src/index.js")
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());

  let asset = &compiler.compilation.assets().get("main.js").unwrap();
  assert_eq!(asset.source.as_ref().unwrap().source(), "console.log(123);");
}

#[tokio::test(flavor = "multi_thread")]
async fn basic_sourcemap() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/basic"))
    .entry("main", "./src/index.js")
    .devtool(Devtool::SourceMap)
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());

  let asset = &compiler.compilation.assets().get("main.js").unwrap();
  assert_eq!(
    asset.source.as_ref().unwrap().source(),
    "console.log(123);\n//# sourceMappingURL=main.js.map"
  );
  assert!(compiler.compilation.assets().get("main.js.map").is_some());
}
