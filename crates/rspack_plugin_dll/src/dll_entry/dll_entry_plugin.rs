use std::sync::Arc;

use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerMake, CompilerOptions,
  Context, DependencyType, EntryOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::{dll_entry_dependency::DllEntryDependency, dll_module_factory};

#[derive(Debug, Clone, Default)]
pub struct DllEntryPluginOptions {
  pub name: String,

  pub context: Context,

  pub entries: Vec<String>,
}

#[plugin]
#[derive(Debug)]
pub struct DllEntryPlugin {
  options: DllEntryPluginOptions,
}

impl DllEntryPlugin {
  pub fn new(options: DllEntryPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[async_trait::async_trait]
impl Plugin for DllEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.DllEntryPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));

    ctx.context.compiler_hooks.make.tap(make::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for DllEntryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::DllEntry,
    Arc::new(dll_module_factory::DllModuleFactory),
  );

  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());

  Ok(())
}

#[plugin_hook(CompilerMake for DllEntryPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  compilation
    .add_entry(
      Box::new(DllEntryDependency::new(&self.options)),
      EntryOptions {
        name: Some(self.options.name.clone()),
        ..Default::default()
      },
    )
    .await?;

  Ok(())
}
