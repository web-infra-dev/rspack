#![feature(let_chains)]

use rspack_core::{
  Compilation, EntryDependency, EntryOptions, MakeParam, Plugin, PluginContext,
  PluginMakeHookOutput,
};

#[derive(Debug)]
pub struct EntryPlugin {
  name: String,
  options: EntryOptions,
  entry_request: String,
}

impl EntryPlugin {
  pub fn new(name: String, entry_request: String, options: EntryOptions) -> Self {
    Self {
      name,
      options,
      entry_request,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for EntryPlugin {
  async fn make(
    &self,
    _ctx: PluginContext,
    compilation: &mut Compilation,
    param: &mut MakeParam,
  ) -> PluginMakeHookOutput {
    if let Some(state) = compilation.options.get_incremental_rebuild_make_state() && !state.is_first() {
      return Ok(());
    }
    let dependency = Box::new(EntryDependency::new(self.entry_request.clone()));
    let dependency_id = compilation.module_graph.add_dependency(dependency);
    compilation.add_entry(dependency_id, self.name.clone(), self.options.clone());
    param.add_force_build_dependency(dependency_id, None);
    Ok(())
  }
}
