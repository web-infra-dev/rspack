use std::sync::Arc;

use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  DependencyType, ExternalType, ModuleExt, ModuleFactoryCreateData, NormalModuleFactoryFactorize,
  Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::runtime::{
  get_federation_global, NormalizedRuntimeInitOptionsWithOutShared, RuntimeGlobals, RuntimeModule,
  Template,
};
use rspack_plugin_javascript::Template;

use crate::utils::normalize_webpack_path;

#[derive(Debug)]
pub struct FederationRuntimeModule {
  runtime_requirements: Set<String>,
  container_name: String,
  init_options_without_shared: NormalizedRuntimeInitOptionsWithOutShared,
}

impl FederationRuntimeModule {
  pub fn new(
    runtime_requirements: Set<String>,
    container_name: String,
    init_options_without_shared: NormalizedRuntimeInitOptionsWithOutShared,
  ) -> Self {
    Self {
      runtime_requirements,
      container_name,
      init_options_without_shared,
    }
  }
}

impl RuntimeModule for FederationRuntimeModule {
  fn name(&self) -> Identifier {
    Identifier::from("federation runtime")
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal - 1
  }

  fn generate(&self, _compilation: &Compilation) -> Result<BoxSource> {
    Ok(Box::new(RawSource::from(Template::asString(vec![
      get_federation_global(Template, RuntimeGlobals, &self.init_options_without_shared),
    ]))))
  }
}
