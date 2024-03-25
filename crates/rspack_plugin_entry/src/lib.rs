#![feature(let_chains)]

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, BoxDependency, Compilation, CompilationParams, CompilerOptions, Context,
  DependencyType, EntryDependency, EntryOptions, MakeParam, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries2};

#[plugin]
#[derive(Debug)]
pub struct EntryPlugin {
  options: EntryOptions,
  entry_request: String,
  context: Context,
}

impl EntryPlugin {
  pub fn new(context: Context, entry_request: String, options: EntryOptions) -> Self {
    Self::new_inner(options, entry_request, context)
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for EntryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  Ok(())
}

#[plugin_hook(AsyncSeries2<Compilation, Vec<MakeParam>> for EntryPlugin)]
async fn make(&self, compilation: &mut Compilation, params: &mut Vec<MakeParam>) -> Result<()> {
  if let Some(state) = compilation.options.get_incremental_rebuild_make_state()
    && !state.is_first()
  {
    return Ok(());
  }
  let this = &self.inner;
  let dependency: BoxDependency = Box::new(EntryDependency::new(
    this.entry_request.clone(),
    this.context.clone(),
  ));
  let dependency_id = *dependency.id();
  compilation.add_entry(dependency, this.options.clone())?;

  params.push(MakeParam::new_force_build_dep_param(dependency_id, None));
  Ok(())
}

#[async_trait]
impl Plugin for EntryPlugin {
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
