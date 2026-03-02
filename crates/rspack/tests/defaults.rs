use rspack::builder::{BuilderContext, CompilerOptionsBuilder};
use rspack_core::Mode;
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
    "        colors: true,\n    },\n    cache",
    "        colors: <env-dependent>,\n    },\n    cache",
  );
  settings.add_filter(
    "        colors: false,\n    },\n    cache",
    "        colors: <env-dependent>,\n    },\n    cache",
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
