//! # FederationModulesPlugin
//!
//! Central hook management system for Module Federation. Provides compilation-specific hooks
//! that allow federation plugins to communicate and share dependency information through
//! a publish-subscribe pattern.

use std::{
  collections::HashMap,
  sync::{Arc, Mutex, OnceLock},
};

use rspack_core::{
  Compilation, CompilationId, CompilationParams, CompilerCompilation, Dependency, Plugin,
};
use rspack_error::Result;
use rspack_hook::{define_hook, plugin, plugin_hook};
#[cfg(allocative)]
use rspack_util::allocative;
use tokio::sync::Mutex as TokioMutex;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  federation_runtime_dependency::FederationRuntimeDependency,
};

define_hook!(AddContainerEntryDependencyHook: Series(dependency: &ContainerEntryDependency));
define_hook!(AddFederationRuntimeDependencyHook: Series(dependency: &FederationRuntimeDependency));
define_hook!(AddRemoteDependencyHook: Series(dependency: &dyn Dependency));

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct FederationModulesPluginCompilationHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub add_container_entry_dependency: Arc<TokioMutex<AddContainerEntryDependencyHookHook>>,
  #[cfg_attr(allocative, allocative(skip))]
  pub add_federation_runtime_dependency: Arc<TokioMutex<AddFederationRuntimeDependencyHookHook>>,
  #[cfg_attr(allocative, allocative(skip))]
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

#[cfg_attr(allocative, allocative::root)]
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

#[plugin_hook(CompilerCompilation for FederationModulesPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  Ok(())
}

impl Plugin for FederationModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.container.FederationModulesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    if let Some(map) = FEDERATION_MODULES_PLUGIN_HOOKS_MAP.get()
      && let Ok(mut map) = map.lock()
    {
      map.remove(&id);
    }
  }
}
