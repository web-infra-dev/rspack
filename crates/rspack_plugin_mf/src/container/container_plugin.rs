use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, ChunkUkey, CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerMake,
  CompilerOptions,
};
use rspack_core::{
  Compilation, CompilationParams, Dependency, DependencyType, EntryOptions, EntryRuntime,
  FilenameTemplate, LibraryOptions, MakeParam, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
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
  pub filename: Option<FilenameTemplate>,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub enhanced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExposeOptions {
  pub name: Option<String>,
  pub import: Vec<String>,
}

#[plugin]
#[derive(Debug)]
pub struct ContainerPlugin {
  options: ContainerPluginOptions,
}

impl ContainerPlugin {
  pub fn new(options: ContainerPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for ContainerPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ContainerEntry,
    Arc::new(ContainerEntryModuleFactory),
  );
  compilation.set_dependency_factory(
    DependencyType::ContainerExposed,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilerMake for ContainerPlugin)]
async fn make(&self, compilation: &mut Compilation, params: &mut Vec<MakeParam>) -> Result<()> {
  let dep = ContainerEntryDependency::new(
    self.options.name.clone(),
    self.options.exposes.clone(),
    self.options.share_scope.clone(),
    self.options.enhanced,
  );
  let dependency_id = *dep.id();
  compilation
    .add_entry(
      Box::new(dep),
      EntryOptions {
        name: Some(self.options.name.clone()),
        runtime: self.options.runtime.clone(),
        filename: self.options.filename.clone(),
        library: Some(self.options.library.clone()),
        ..Default::default()
      },
    )
    .await?;

  params.push(MakeParam::new_force_build_dep_param(dependency_id, None));
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ContainerPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE) {
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    if self.options.enhanced {
      compilation.add_runtime_module(chunk_ukey, Box::new(ExposeRuntimeModule::new()))?;
    }
  }
  Ok(None)
}

#[async_trait]
impl Plugin for ContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerPlugin"
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
    ctx.context.compiler_hooks.make.tap(make::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
