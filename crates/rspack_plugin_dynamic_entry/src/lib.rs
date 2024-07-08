use async_trait::async_trait;
use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, BoxDependency, Compilation, CompilationParams, CompilerCompilation, CompilerMake,
  CompilerOptions, Context, DependencyType, EntryDependency, EntryOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

pub struct EntryDynamicResult {
  pub import: Vec<String>,
  pub options: EntryOptions,
}

type EntryDynamic =
  Box<dyn for<'a> Fn() -> BoxFuture<'static, Result<Vec<EntryDynamicResult>>> + Sync + Send>;

pub struct DynamicEntryPluginOptions {
  pub context: Context,
  pub entry: EntryDynamic,
}

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct DynamicEntryPlugin {
  context: Context,
  #[derivative(Debug = "ignore")]
  entry: EntryDynamic,
}

impl DynamicEntryPlugin {
  pub fn new(options: DynamicEntryPluginOptions) -> Self {
    Self::new_inner(options.context, options.entry)
  }
}

#[plugin_hook(CompilerCompilation for DynamicEntryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  Ok(())
}

#[plugin_hook(CompilerMake for DynamicEntryPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let entry_fn = &self.entry;
  let decs = entry_fn().await?;
  for EntryDynamicResult { import, options } in decs {
    for entry in import {
      let dependency: BoxDependency =
        Box::new(EntryDependency::new(entry, self.context.clone(), false));
      compilation.add_entry(dependency, options.clone()).await?;
    }
  }
  Ok(())
}

#[async_trait]
impl Plugin for DynamicEntryPlugin {
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
    Ok(())
  }
}
