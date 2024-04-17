use std::sync::Arc;

use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

use crate::federation_runtime_module::FederationRuntimeModule;
use crate::utils::{get_federation_global_scope, normalize_runtime_init_options_without_shared};

#[derive(Debug, Serialize)]
pub struct FederationRuntimePluginOptions {
  pub namex: Option<String>,
  // Other options can be added here as needed
}

#[plugin]
#[derive(Debug)]
pub struct FederationRuntimePlugin {
  options: FederationRuntimePluginOptions,
  entry_file_path: String,
  bundler_runtime_path: String,
}

impl FederationRuntimePlugin {
  pub fn new(options: FederationRuntimePluginOptions) -> Self {
    Self { options }
  }

  fn inject_runtime(&self, compilation: &mut Compilation) -> Result<()> {
    if let Some(name) = &self.options.name {
      let init_options_without_shared =
        normalize_runtime_init_options_without_shared(&self.options);
      let federation_global = get_federation_global_scope(RuntimeGlobals);

      compilation.hooks.additional_tree_runtime_requirements.tap(
        self.constructor_name(),
        move |chunk, runtime_requirements| {
          if !runtime_requirements.contains(&federation_global) {
            runtime_requirements.insert(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
            runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
            runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
            runtime_requirements.insert(federation_global);
            compilation.add_runtime_module(
              chunk,
              Box::new(FederationRuntimeModule::new(
                runtime_requirements.clone(),
                name.clone(),
                init_options_without_shared.clone(),
              )),
            )?;
          }
          Ok(())
        },
      );
    }
    Ok(())
  }

  fn constructor_name(&self) -> &'static str {
    "FederationRuntimePlugin"
  }
}

#[plugin_hook(CompilerCompilation for FederationRuntimePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.inject_runtime(compilation)
}

#[async_trait]
impl Plugin for FederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.FederationRuntimePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}
