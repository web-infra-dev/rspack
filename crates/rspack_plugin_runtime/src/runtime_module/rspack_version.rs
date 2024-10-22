use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RspackVersionRuntimeModule {
  id: Identifier,
  version: String,
}

impl RspackVersionRuntimeModule {
  pub fn new(version: String) -> Self {
    Self::with_default(Identifier::from("webpack/runtime/rspack_version"), version)
  }
}

impl RuntimeModule for RspackVersionRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<String> {
    Ok(
      include_str!("runtime/get_version.js")
        .cow_replace("$VERSION$", &self.version)
        .to_string(),
    )
  }
}
