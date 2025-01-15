use rspack::builder::Builder as _;
use rspack_core::{Compiler, CompilerOptions, EntryDescription, Optimization};
use rspack_paths::Utf8Path;

#[tokio::test(flavor = "multi_thread")]
async fn basic() {
  let dir = Utf8Path::new(env!("CARGO_MANIFEST_DIR"));
  let mut compiler = Compiler::builder()
    .compiler_options(
      CompilerOptions::builder()
        .context(Utf8Path::new(dir).join("tests/fixtures/basic"))
        .entry(
          "main".to_string(),
          EntryDescription {
            import: vec!["./src/index.js".to_string()],
            ..Default::default()
          },
        )
        .optimization(Optimization::builder().node_env("'development'".to_string())),
    )
    .build();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());

  let asset = &compiler.compilation.assets().get("main.js").unwrap();
  assert_eq!(asset.source.as_ref().unwrap().source(), "console.log(123);");
}
