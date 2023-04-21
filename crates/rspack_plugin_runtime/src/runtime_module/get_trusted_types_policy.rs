use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct GetTrustedTypesPolicyRuntimeModule {
  id: Identifier,
}

impl Default for GetTrustedTypesPolicyRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_trusted_types_policy"),
    }
  }
}

impl RuntimeModule for GetTrustedTypesPolicyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let trusted_types = compilation
      .options
      .output
      .trusted_types
      .as_ref()
      .expect("should have trusted_types");
    RawSource::from(include_str!("runtime/get_trusted_types_policy.js").replace(
      "$policyName$",
      &trusted_types.policy_name.clone().unwrap_or_default(),
    ))
    .boxed()
  }
}

impl_runtime_module!(GetTrustedTypesPolicyRuntimeModule);
