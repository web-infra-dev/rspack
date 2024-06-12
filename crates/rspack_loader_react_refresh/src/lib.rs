use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};

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
    assert!(identifier.starts_with(REACT_REFRESH_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

#[async_trait::async_trait]
impl Loader<RunnerContext> for ReactRefreshLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Ok(());
    };
    let mut source = content.try_into_string()?;
    source += r#"
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
    loader_context.content = Some(source.into());
    Ok(())
  }
}

pub const REACT_REFRESH_LOADER_IDENTIFIER: &str = "builtin:react-refresh-loader";

impl Identifiable for ReactRefreshLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
