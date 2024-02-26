#![feature(let_chains)]

use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, BoxDependency, Compilation, CompilationParams, CompilerOptions, Context,
  DependencyType, EntryDependency, EntryOptions, MakeParam, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::AsyncSeries2;

#[derive(Debug)]
pub struct EntryPlugin {
  inner: Arc<EntryPluginInner>,
}

impl EntryPlugin {
  pub fn new(context: Context, entry_request: String, options: EntryOptions) -> Self {
    Self {
      inner: Arc::new(EntryPluginInner::new(context, entry_request, options)),
    }
  }
}

#[derive(Debug)]
pub struct EntryPluginInner {
  options: EntryOptions,
  entry_request: String,
  context: Context,
}

impl EntryPluginInner {
  pub fn new(context: Context, entry_request: String, options: EntryOptions) -> Self {
    Self {
      options,
      entry_request,
      context,
    }
  }
}

struct EntryPluginCompilationHook;

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for EntryPluginCompilationHook {
  async fn run(&self, compilation: &mut Compilation, params: &mut CompilationParams) -> Result<()> {
    compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
    Ok(())
  }
}

struct EntryPluginMakeHook {
  inner: Arc<EntryPluginInner>,
}

#[async_trait]
impl AsyncSeries2<Compilation, Vec<MakeParam>> for EntryPluginMakeHook {
  async fn run(&self, compilation: &mut Compilation, params: &mut Vec<MakeParam>) -> Result<()> {
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
      .tap(Box::new(EntryPluginCompilationHook));
    ctx
      .context
      .compiler_hooks
      .make
      .tap(Box::new(EntryPluginMakeHook {
        inner: self.inner.clone(),
      }));
    Ok(())
  }
}
