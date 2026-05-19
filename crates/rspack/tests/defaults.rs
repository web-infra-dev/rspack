use rspack::builder::{BuilderContext, CompilerOptionsBuilder, OutputOptionsBuilder};
use rspack_core::{LibraryOptions, Mode};
use supports_color::Stream;

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
  // stats.colors defaults to env-dependent (TTY / FORCE_COLOR / NO_COLOR)
  settings.add_filter(
    r"(?m)^\s*colors: (true|false),$",
    "        colors: <env-dependent>,",
  );
  settings.bind(|| {
    insta::assert_debug_snapshot!(options);
    insta::assert_debug_snapshot!(builder_context);
  });
}

/// Default stats.colors must follow environment color support (issue #9353).
#[tokio::test(flavor = "multi_thread")]
async fn default_stats_colors_follows_environment() {
  let mut builder_context = BuilderContext::default();
  let options = CompilerOptionsBuilder::default()
    .mode(Mode::None)
    .build(&mut builder_context)
    .unwrap();
  let expected = supports_color::on(Stream::Stdout).is_some();
  assert_eq!(
    options.stats.colors, expected,
    "stats.colors default should match environment color support"
  );
}

#[tokio::test(flavor = "multi_thread")]
async fn modern_module_library_keeps_module_import_externals_default() {
  let mut builder_context = BuilderContext::default();
  CompilerOptionsBuilder::default()
    .mode(Mode::None)
    .output(
      OutputOptionsBuilder::default()
        .module(true)
        .library(LibraryOptions {
          name: None,
          export: None,
          library_type: "modern-module".to_string(),
          umd_named_define: None,
          auxiliary_comment: None,
          amd_container: None,
        }),
    )
    .externals("react".to_string().into())
    .build(&mut builder_context)
    .unwrap();

  let builder_context = format!("{builder_context:?}");
  assert!(
    builder_context.contains(r#"ExternalsPlugin(("module-import""#),
    "modern-module libraries should keep the module-import externals default until the next major"
  );
  assert!(
    !builder_context.contains(r#"ExternalsPlugin(("modern-module""#),
    "modern-module externals should require an explicit externals_type opt-in"
  );
}
