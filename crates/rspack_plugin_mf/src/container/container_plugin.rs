use std::sync::Arc;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerMake, DependencyRef,
  DependencyType, EntryOptions, EntryRuntime, Filename, LibraryOptions, ModuleLayer, Plugin,
  RuntimeGlobals, RuntimeModule, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  container_entry_module_factory::ContainerEntryModuleFactory,
  expose_runtime_module::ExposeRuntimeModule, federation_modules_plugin::FederationModulesPlugin,
};
use crate::ShareScope;

#[derive(Debug)]
pub struct ContainerPluginOptions {
  pub name: String,
  pub share_scope: ShareScope,
  pub library: LibraryOptions,
  pub runtime: Option<EntryRuntime>,
  pub filename: Option<Filename>,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub enhanced: bool,
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct ExposeOptions {
  pub name: Option<String>,
  pub import: Vec<String>,
}

#[plugin]
#[derive(Debug)]
pub struct ContainerPlugin {
  options: ContainerPluginOptions,
  expose_layers: Vec<Option<ModuleLayer>>,
}

impl ContainerPlugin {
  pub fn new(options: ContainerPluginOptions) -> Self {
    let expose_layers = vec![None; options.exposes.len()];
    Self::new_inner(options, expose_layers)
  }

  pub fn new_with_expose_layers(
    options: ContainerPluginOptions,
    mut expose_layers: Vec<Option<ModuleLayer>>,
  ) -> Self {
    expose_layers.resize(options.exposes.len(), None);
    expose_layers.truncate(options.exposes.len());
    Self::new_inner(options, expose_layers)
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
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let dep = ContainerEntryDependency::new_with_expose_layers(
    self.options.name.clone(),
    self.options.exposes.clone(),
    self.expose_layers.clone(),
    self.options.share_scope.clone(),
    self.options.enhanced,
  );

  // Call federation hook for dependency tracking
  let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);
  hooks
    .add_container_entry_dependency
    .lock()
    .await
    .call(&dep)
    .await?;

  compilation
    .add_entry(
      DependencyRef::new(dep),
      EntryOptions {
        name: Some(self.options.name.clone()),
        runtime: self.options.runtime.clone(),
        filename: self.options.filename.clone(),
        library: Some(self.options.library.clone()),
        ..Default::default()
      },
    )
    .await?;
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ContainerPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let Some(entry_options) = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
    .and_then(|chunk| {
      chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    })
  else {
    return Ok(());
  };
  if matches!(&entry_options.name, Some(name) if name == &self.options.name)
    && compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .has_chunk_module_by_source_type(
        chunk_ukey,
        SourceType::Expose,
        compilation.get_module_graph(),
      )
    && compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .has_chunk_entry_dependent_chunks(
        chunk_ukey,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      )
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES);
  }
  Ok(())
}
