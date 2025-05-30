use std::{
  collections::HashMap,
  sync::{Arc, Mutex, OnceLock},
};

use rspack_core::{
  ApplyContext, Compilation, CompilationId, CompilationParams, CompilerCompilation,
  CompilerOptions, Dependency, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{define_hook, plugin, plugin_hook};
use tokio::sync::Mutex as TokioMutex;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  federation_runtime_dependency::FederationRuntimeDependency,
};

// Hook definitions
define_hook!(AddContainerEntryDependencyHook: Series(dependency: &ContainerEntryDependency));
define_hook!(AddFederationRuntimeDependencyHook: Series(dependency: &FederationRuntimeDependency));
define_hook!(AddRemoteDependencyHook: Series(dependency: &dyn Dependency));

// Struct to hold instances of the hooks for a specific compilation
pub struct FederationModulesPluginCompilationHooks {
  pub add_container_entry_dependency: Arc<TokioMutex<AddContainerEntryDependencyHookHook>>,
  pub add_federation_runtime_dependency: Arc<TokioMutex<AddFederationRuntimeDependencyHookHook>>,
  pub add_remote_dependency: Arc<TokioMutex<AddRemoteDependencyHookHook>>,
}

impl Default for FederationModulesPluginCompilationHooks {
  fn default() -> Self {
    Self {
      add_container_entry_dependency: Arc::new(TokioMutex::new(
        AddContainerEntryDependencyHookHook::default(),
      )),
      add_federation_runtime_dependency: Arc::new(TokioMutex::new(
        AddFederationRuntimeDependencyHookHook::default(),
      )),
      add_remote_dependency: Arc::new(TokioMutex::new(AddRemoteDependencyHookHook::default())),
    }
  }
}

// Static map to store hooks per CompilationId
static FEDERATION_MODULES_PLUGIN_HOOKS_MAP: OnceLock<
  Mutex<HashMap<CompilationId, Arc<FederationModulesPluginCompilationHooks>>>,
> = OnceLock::new();

#[plugin]
#[derive(Debug, Default)]
pub struct FederationModulesPlugin;

impl FederationModulesPlugin {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self::new_inner()
  }

  pub fn get_compilation_hooks(
    compilation: &Compilation,
  ) -> Arc<FederationModulesPluginCompilationHooks> {
    let key = compilation.id();
    let mut map = FEDERATION_MODULES_PLUGIN_HOOKS_MAP
      .get_or_init(|| Mutex::new(HashMap::new()))
      .lock()
      .expect("Failed to lock FEDERATION_MODULES_PLUGIN_HOOKS_MAP");
    map
      .entry(key)
      .or_insert_with(|| Arc::new(FederationModulesPluginCompilationHooks::default()))
      .clone()
  }

  pub fn get_compilation_hooks_by_id(
    compilation_id: CompilationId,
  ) -> Arc<FederationModulesPluginCompilationHooks> {
    let mut map = FEDERATION_MODULES_PLUGIN_HOOKS_MAP
      .get_or_init(|| Mutex::new(HashMap::new()))
      .lock()
      .expect("Failed to lock FEDERATION_MODULES_PLUGIN_HOOKS_MAP");
    map
      .entry(compilation_id)
      .or_insert_with(|| Arc::new(FederationModulesPluginCompilationHooks::default()))
      .clone()
  }
}

// Hook into the compilation process
#[plugin_hook(CompilerCompilation for FederationModulesPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  Ok(())
}

#[async_trait::async_trait]
impl Plugin for FederationModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.container.FederationModulesPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}
