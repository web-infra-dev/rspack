use std::sync::Arc;

use once_cell::sync::OnceCell;
use rspack_core::{CompilerOptions, CssModuleGeneratorOptions, ModuleType, ResourceData};
use rspack_error::Result;

use crate::utils::{LocalIdentModuleHashOptions, LocalIdentOptions};

/// Lazily initializes the shared inputs used to generate local identifiers for
/// one CSS module. Identifiers are still generated during parsing.
pub(super) struct CssLocalIdents<'context> {
  options: OnceCell<LocalIdentOptions<'context>>,
  resource_data: &'context ResourceData,
  module_type: &'context ModuleType,
  source: Arc<str>,
  compiler_options: &'context CompilerOptions,
  generator_options: &'context CssModuleGeneratorOptions,
}

impl<'context> CssLocalIdents<'context> {
  pub(super) fn new(
    resource_data: &'context ResourceData,
    module_type: &'context ModuleType,
    source: Arc<str>,
    compiler_options: &'context CompilerOptions,
    generator_options: &'context CssModuleGeneratorOptions,
  ) -> Self {
    Self {
      options: OnceCell::new(),
      resource_data,
      module_type,
      source,
      compiler_options,
      generator_options,
    }
  }

  pub(super) async fn generate(
    &self,
    local: &str,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<String> {
    self
      .options
      .get_or_init(|| {
        LocalIdentOptions::new(
          self.resource_data,
          self.module_type,
          self.source.clone(),
          self.compiler_options,
          self.generator_options,
        )
      })
      .get_local_ident(local, module_hash_options)
      .await
  }
}
