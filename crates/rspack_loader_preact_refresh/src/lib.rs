mod plugin;

pub use plugin::PreactRefreshLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  RunnerContext,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_loader_runner::{Identifier, Loader, LoaderContext};

#[cacheable]
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

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for PreactRefreshLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_source() else {
      return Ok(());
    };
    let source = ConcatSource::new([
      content,
      RawStringSource::from(concat!("\n", include_str!("runtime.js"))).boxed(),
    ])
    .boxed();
    let additional_data = loader_context.take_additional_data();
    loader_context.finish_with((source, additional_data));
    Ok(())
  }
}

pub const PREACT_REFRESH_LOADER_IDENTIFIER: &str = "builtin:preact-refresh-loader";
