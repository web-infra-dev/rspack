use std::sync::Arc;

use rspack_core::{
  Compilation, CompilationParams, CompilerCompilation, CompilerMake, DependencyType, Filename,
  Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::{
  share_container_entry_dependency::ShareContainerEntryDependency,
  share_container_entry_module_factory::ShareContainerEntryModuleFactory,
};

#[derive(Debug)]
pub struct ShareContainerPluginOptions {
  pub name: String,
  pub share_name: String,
  pub request: String,
  pub version: String,
  pub global_name: String,
  pub file_name: Option<Filename>,
}

#[plugin]
#[derive(Debug)]
pub struct ShareContainerPlugin {
  options: ShareContainerPluginOptions,
}

impl ShareContainerPlugin {
  pub fn new(options: ShareContainerPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for ShareContainerPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ShareContainerEntry,
    Arc::new(ShareContainerEntryModuleFactory),
  );
  compilation.set_dependency_factory(
    DependencyType::ShareContainerFallback,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilerMake for ShareContainerPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let dep = ShareContainerEntryDependency::new(
    self.options.name.clone(),
    self.options.share_name.clone(),
    self.options.request.clone(),
    self.options.version.clone(),
    self.options.global_name.clone(),
  );

  compilation
    .add_entry(
      Box::new(dep),
      rspack_core::EntryOptions {
        name: Some(self.options.name.clone()),
        filename: self.options.file_name.clone(),
        ..Default::default()
      },
    )
    .await?;
  Ok(())
}

impl Plugin for ShareContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareContainerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}
