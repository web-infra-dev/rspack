use std::sync::Arc;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements, CompilationParams,
  CompilerCompilation, CompilerMake, DependencyType, Filename, LibraryOptions, Plugin,
  RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  container::{
    container_entry_dependency::ContainerEntryDependency,
    container_entry_module_factory::ContainerEntryModuleFactory,
  },
  sharing::shared_container_runtime_module::ShareContainerRuntimeModule,
};

#[derive(Debug)]
pub struct SharedContainerPluginOptions {
  pub name: String,
  pub request: String,
  pub version: String,
  pub file_name: Option<Filename>,
  pub library: LibraryOptions,
}

#[plugin]
#[derive(Debug)]
pub struct SharedContainerPlugin {
  options: SharedContainerPluginOptions,
}

impl SharedContainerPlugin {
  pub fn new(options: SharedContainerPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for SharedContainerPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ShareContainerEntry,
    Arc::new(ContainerEntryModuleFactory),
  );
  compilation.set_dependency_factory(
    DependencyType::ShareContainerFallback,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilerMake for SharedContainerPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let dep = ContainerEntryDependency::new_share_container_entry(
    self.options.name.clone(),
    self.options.request.clone(),
    self.options.version.clone(),
  );

  compilation
    .add_entry(
      Box::new(dep),
      rspack_core::EntryOptions {
        name: Some(self.options.name.clone()),
        filename: self.options.file_name.clone(),
        library: Some(self.options.library.clone()),
        ..Default::default()
      },
    )
    .await?;
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for SharedContainerPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if let Some(name) = chunk.name()
    && name == self.options.name
  {
    compilation.add_runtime_module(chunk_ukey, ShareContainerRuntimeModule::new().boxed())?;
  }
  Ok(())
}

impl Plugin for SharedContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.SharedContainerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}
