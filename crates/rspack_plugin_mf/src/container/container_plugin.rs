use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Compilation, CompilationArgs, CompilationParams,
  Dependency, DependencyType, EntryOptions, EntryRuntime, Filename, LibraryOptions, MakeParam,
  Plugin, PluginAdditionalChunkRuntimeRequirementsOutput, PluginCompilationHookOutput,
  PluginContext, PluginMakeHookOutput, RuntimeGlobals,
};
use serde::Serialize;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  container_entry_module_factory::ContainerEntryModuleFactory,
  expose_runtime_module::ExposeRuntimeModule,
};

#[derive(Debug)]
pub struct ContainerPluginOptions {
  pub name: String,
  pub share_scope: String,
  pub library: LibraryOptions,
  pub runtime: Option<EntryRuntime>,
  pub filename: Option<Filename>,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub enhanced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExposeOptions {
  pub name: Option<String>,
  pub import: Vec<String>,
}

#[derive(Debug)]
pub struct ContainerPlugin {
  options: ContainerPluginOptions,
}

impl ContainerPlugin {
  pub fn new(options: ContainerPluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait]
impl Plugin for ContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerPlugin"
  }

  async fn compilation(
    &self,
    args: CompilationArgs<'_>,
    params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    args.compilation.set_dependency_factory(
      DependencyType::ContainerEntry,
      Arc::new(ContainerEntryModuleFactory),
    );
    args.compilation.set_dependency_factory(
      DependencyType::ContainerExposed,
      params.normal_module_factory.clone(),
    );
    Ok(())
  }

  async fn make(
    &self,
    _ctx: PluginContext,
    compilation: &mut Compilation,
    param: &mut MakeParam,
  ) -> PluginMakeHookOutput {
    let dep = ContainerEntryDependency::new(
      self.options.name.clone(),
      self.options.exposes.clone(),
      self.options.share_scope.clone(),
    );
    let dependency_id = *dep.id();
    compilation.add_entry(
      Box::new(dep),
      EntryOptions {
        name: Some(self.options.name.clone()),
        runtime: self.options.runtime.clone(),
        filename: self.options.filename.clone(),
        library: Some(self.options.library.clone()),
        ..Default::default()
      },
    );
    param.add_force_build_dependency(dependency_id, None);
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE)
    {
      args
        .runtime_requirements
        .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      args.compilation.add_runtime_module(
        args.chunk,
        Box::new(ExposeRuntimeModule::new(self.options.enhanced)),
      );
    }
    Ok(())
  }
}
