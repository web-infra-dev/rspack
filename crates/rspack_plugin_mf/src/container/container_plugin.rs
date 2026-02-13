use std::sync::Arc;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerMake, DependencyType,
  EntryOptions, EntryRuntime, Filename, LibraryOptions, Plugin, RuntimeGlobals, RuntimeModule,
  SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  container_entry_module_factory::ContainerEntryModuleFactory,
  expose_runtime_module::ExposeRuntimeModule, federation_modules_plugin::FederationModulesPlugin,
};

#[derive(Debug)]
pub struct ContainerPluginOptions {
  pub name: String,
  pub share_scope: Vec<String>,
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
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let dep = ContainerEntryDependency::new(
    self.options.name.clone(),
    self.options.exposes.clone(),
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

#[plugin_hook(CompilationRuntimeRequirementInTree for ContainerPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE) {
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    if self.options.enhanced {
      runtime_modules_to_add.push((
        *chunk_ukey,
        Box::new(ExposeRuntimeModule::new(&compilation.runtime_template)),
      ));
    }
  }
  Ok(None)
}

impl Plugin for ContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
