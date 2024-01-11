#![feature(let_chains)]

use rspack_core::{
  BoxDependency, Compilation, CompilationArgs, CompilationParams, Context, DependencyType,
  EntryDependency, EntryOptions, MakeParam, Plugin, PluginCompilationHookOutput, PluginContext,
  PluginMakeHookOutput,
};

#[derive(Debug)]
pub struct EntryPlugin {
  options: EntryOptions,
  entry_request: String,
  context: Context,
}

impl EntryPlugin {
  pub fn new(context: Context, entry_request: String, options: EntryOptions) -> Self {
    Self {
      options,
      entry_request,
      context,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for EntryPlugin {
  async fn compilation(
    &self,
    args: CompilationArgs<'_>,
    params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    args
      .compilation
      .set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
    Ok(())
  }

  async fn make(
    &self,
    _ctx: PluginContext,
    compilation: &mut Compilation,
    params: &mut Vec<MakeParam>,
  ) -> PluginMakeHookOutput {
    if let Some(state) = compilation.options.get_incremental_rebuild_make_state()
      && !state.is_first()
    {
      return Ok(());
    }
    let dependency: BoxDependency = Box::new(EntryDependency::new(
      self.entry_request.clone(),
      self.context.clone(),
    ));
    let dependency_id = *dependency.id();
    compilation.add_entry(dependency, self.options.clone())?;

    params.push(MakeParam::new_force_build_dep_param(dependency_id, None));
    Ok(())
  }
}
