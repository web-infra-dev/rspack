use rspack::builder::{BuilderContext, CompilerOptionsBuilder};
use rspack_core::Mode;

#[tokio::test(flavor = "multi_thread")]
async fn default_options() {
  let mut builder_context = BuilderContext::default();
  let options = CompilerOptionsBuilder::default()
    .mode(Mode::None)
    .build(&mut builder_context)
    .unwrap();
  let cwd = std::env::current_dir().unwrap();

  let mut settings = insta::Settings::clone_current();
  settings.add_filter(&cwd.to_string_lossy(), "<cwd>");
  settings.bind(|| {
    insta::assert_debug_snapshot!(options);
    insta::assert_debug_snapshot!(builder_context);
  });
}
