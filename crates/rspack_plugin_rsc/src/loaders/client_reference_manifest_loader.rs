use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Loader, LoaderContext};

pub const CLIENT_REFERENCE_MANIFEST_LOADER_IDENTIFIER: &str =
  "builtin:rsc-client-reference-manifest-loader";

#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ClientReferenceManifestLoader {
  identifier: Identifier,
}

impl ClientReferenceManifestLoader {
  pub fn new() -> Self {
    Self {
      identifier: CLIENT_REFERENCE_MANIFEST_LOADER_IDENTIFIER.into(),
    }
  }

  pub fn with_identifier<T: Into<Identifier>>(mut self, identifier: T) -> Self {
    let identifier = identifier.into();
    assert!(identifier.starts_with(CLIENT_REFERENCE_MANIFEST_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for ClientReferenceManifestLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:rsc-client-reference-manifest-loader", skip_all, fields(
    perfetto.track_name = "loader:rsc-client-reference-manifest-loader",
    perfetto.process_name = "Loader Analysis",
    resource = loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let code = "export default __RSPACK_RSC_CLIENT_REFERENCE_MANIFEST__".to_string();

    loader_context.finish_with(code);

    Ok(())
  }
}
