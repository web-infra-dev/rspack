use async_trait::async_trait;
use serde::Serialize;

use super::container_entry_dependency::ContainerEntryDependency;
use crate::{
  Compilation, Dependency, EntryOptions, EntryRuntime, Filename, LibraryOptions, MakeParam, Plugin,
  PluginContext, PluginMakeHookOutput,
};

#[derive(Debug)]
pub struct ContainerPluginOptions {
  pub name: String,
  pub share_scope: String,
  pub library: LibraryOptions,
  pub runtime: Option<EntryRuntime>,
  pub filename: Option<Filename>,
  pub exposes: Vec<(String, ExposeOptions)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExposeOptions {
  pub name: Option<String>,
  pub import: Vec<String>,
}

#[derive(Debug)]
pub struct ContainerPlugin {
  options: ContainerPluginOptions,
}

impl ContainerPlugin {
  pub fn new(options: ContainerPluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait]
impl Plugin for ContainerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerPlugin"
  }

  async fn make(
    &self,
    _ctx: PluginContext,
    compilation: &mut Compilation,
    param: &mut MakeParam,
  ) -> PluginMakeHookOutput {
    let dep = ContainerEntryDependency::new(
      self.options.name.clone(),
      self.options.exposes.clone(),
      self.options.share_scope.clone(),
    );
    let dependency_id = *dep.id();
    compilation.add_entry(
      Box::new(dep),
      EntryOptions {
        name: Some(self.options.name.clone()),
        runtime: self.options.runtime.clone(),
        filename: self.options.filename.clone(),
        library: Some(self.options.library.clone()),
        ..Default::default()
      },
    );
    param.add_force_build_dependency(dependency_id, None);
    Ok(())
  }
}
