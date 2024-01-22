use rspack_core::LoaderRunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};

pub struct PreactRefreshLoader {
  identifier: Identifier,
}

impl Default for PreactRefreshLoader {
  fn default() -> Self {
    Self {
      identifier: PREACT_REFRESH_LOADER_IDENTIFIER.into(),
    }
  }
}

impl PreactRefreshLoader {
  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:preact-refresh-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(PREACT_REFRESH_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for PreactRefreshLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let content = std::mem::take(&mut loader_context.content).expect("Content should be available");
    let mut source = content.try_into_string()?;
    source += "\n";
    source += include_str!("runtime.js");
    loader_context.content = Some(source.into());
    Ok(())
  }
}

pub const PREACT_REFRESH_LOADER_IDENTIFIER: &str = "builtin:preact-refresh-loader";

impl Identifiable for PreactRefreshLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
