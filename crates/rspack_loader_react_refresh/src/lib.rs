mod plugin;

pub use plugin::ReactRefreshLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  RunnerContext,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
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

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for ReactRefreshLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_source() else {
      return Ok(());
    };
    let supports_arrow_function = loader_context
      .context
      .options
      .output
      .environment
      .supports_arrow_function();

    let runtime = if supports_arrow_function {
      r#"
function $RefreshSig$() { return $ReactRefreshRuntime$.createSignatureFunctionForTransform() }
function $RefreshReg$(type, id) { $ReactRefreshRuntime$.register(type, __webpack_module__.id + "_" + id) }
Promise.resolve().then(() => { $ReactRefreshRuntime$.refresh(__webpack_module__.id, __webpack_module__.hot) });
"#
    } else {
      r#"
function $RefreshSig$() { return $ReactRefreshRuntime$.createSignatureFunctionForTransform() }
function $RefreshReg$(type, id) { $ReactRefreshRuntime$.register(type, __webpack_module__.id + "_" + id) }
Promise.resolve().then(function() { $ReactRefreshRuntime$.refresh(__webpack_module__.id, __webpack_module__.hot) });
"#
    };
    let source =
      ConcatSource::new([content, RawStringSource::from_static(runtime).boxed()]).boxed();
    let additional_data = loader_context.take_additional_data();
    loader_context.finish_with((source, additional_data));
    Ok(())
  }
}

pub const REACT_REFRESH_LOADER_IDENTIFIER: &str = "builtin:react-refresh-loader";
