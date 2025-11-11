mod plugin;

pub use plugin::ReactRefreshLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifier, Loader, LoaderContext};

#[cacheable]
pub struct ReactRefreshLoader {
  identifier: Identifier,
}

impl Default for ReactRefreshLoader {
  fn default() -> Self {
    Self {
      identifier: REACT_REFRESH_LOADER_IDENTIFIER.into(),
    }
  }
}

impl ReactRefreshLoader {
  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:react-refresh-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    debug_assert!(identifier.starts_with(REACT_REFRESH_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

const REFRESH_SNIPPET: &str = r#"
function $RefreshSig$() {
  return $ReactRefreshRuntime$.createSignatureFunctionForTransform();
}
function $RefreshReg$(type, id) {
  $ReactRefreshRuntime$.register(type, __webpack_module__.id + "_" + id);
}
Promise.resolve().then(function() {
  $ReactRefreshRuntime$.refresh(__webpack_module__.id, __webpack_module__.hot);
});
"#;

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for ReactRefreshLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let mut source = content.try_into_string()?;

    source.reserve(REFRESH_SNIPPET.len());
    source.push_str(REFRESH_SNIPPET);

    let sm = loader_context.take_source_map();
    loader_context.finish_with((source, sm));
    Ok(())
  }
}

pub const REACT_REFRESH_LOADER_IDENTIFIER: &str = "builtin:react-refresh-loader";
