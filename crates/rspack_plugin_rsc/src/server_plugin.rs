use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_collections::{Identifiable, IdentifierMap};
use rspack_core::{
  BoxDependency, ChunkUkey, Compilation, CompilationParams, CompilationProcessAssets, ProcessAssetArtifact, CompilationRuntimeRequirementInTree, CompilerDone, CompilerFailed, CompilerFinishMake,
  CompilerThisCompilation, Dependency, DependencyId, EntryDependency, EntryOptions, Logger, Plugin,
  RuntimeGlobals, RuntimeModule, RuntimeSpec, get_entry_runtime,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;

use crate::{
  component_info::{
    ClientComponentImports, CssImports, collect_component_info_from_entry_denendency,
  },
  constants::LAYERS_NAMES,
  coordinator::Coordinator,
  hot_reloader::track_server_component_changes,
  loaders::{
    action_entry_loader::ACTION_ENTRY_LOADER_IDENTIFIER,
    client_entry_loader::CLIENT_ENTRY_LOADER_IDENTIFIER,
  },
  manifest_runtime_module::RscManifestRuntimeModule,
  plugin_state::{ActionIdNamePair, PLUGIN_STATES, PluginState},
};

#[derive(Debug)]
struct ClientEntry {
  entry_name: String,
  runtime: RuntimeSpec,
  client_imports: ClientComponentImports,
  css_imports: CssImports,
}

#[derive(Debug)]
struct InjectedSsrEntry {
  runtime: RuntimeSpec,
  add_entry: (BoxDependency, EntryOptions),
  dependency_id: DependencyId,
}

struct ActionEntry {
  actions: FxHashMap<String, Vec<ActionIdNamePair>>,
  entry_name: String,
  runtime: RuntimeSpec,
  from_client: bool,
}

#[derive(Debug)]
struct InjectedActionEntry {
  pub runtime: RuntimeSpec,
  pub add_entry: (BoxDependency, EntryOptions),
}

type OnServerComponentChanges = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct RscServerPluginOptions {
  pub coordinator: Arc<Coordinator>,
  pub on_server_component_changes: Option<OnServerComponentChanges>,
}

#[plugin]
#[derive(Debug)]
pub struct RscServerPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
  #[debug(skip)]
  on_server_component_changes: Option<OnServerComponentChanges>,
  prev_server_component_hashes: AtomicRefCell<IdentifierMap<u64>>,
}

impl RscServerPlugin {
  pub fn new(options: RscServerPluginOptions) -> Self {
    Self::new_inner(
      options.coordinator,
      options.on_server_component_changes,
      Default::default(),
    )
  }
}

#[plugin_hook(CompilerThisCompilation for RscServerPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,

) -> Result<()> {
  // Initialize or reset the plugin state for the current compilation.
  // If a state already exists, clear it; otherwise, insert a default state.
  match PLUGIN_STATES.entry(compilation.compiler_id()) {
    dashmap::Entry::Occupied(mut occupied_entry) => {
      occupied_entry.get_mut().clear();
    }
    dashmap::Entry::Vacant(vacant_entry) => {
      vacant_entry.insert(Default::default());
    }
  };

  self.coordinator.start_server_entries_compilation().await?;

  Ok(())
}

#[plugin_hook(CompilerFinishMake for RscServerPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.RscServerPlugin");

  {
    let mut plugin_state = PLUGIN_STATES.entry(compilation.compiler_id()).or_default();

    let start = logger.time("track server component changes");
    let mut prev_server_component_hashes = self.prev_server_component_hashes.borrow_mut();
    plugin_state.changed_server_components_per_entry =
      track_server_component_changes(compilation, &mut prev_server_component_hashes);
    logger.time_end(start);
  }

  let start = logger.time("create client entries");
  self.create_client_entries(compilation).await?;
  logger.time_end(start);

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for RscServerPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::RSC_MANIFEST) {
    runtime_modules_to_add.push((
      *chunk_ukey,
      Box::new(RscManifestRuntimeModule::new(&compilation.runtime_template)),
    ));
  }
  Ok(None)
}

#[plugin_hook(CompilationProcessAssets for RscServerPlugin)]
async fn process_assets(&self, compilation: &Compilation, process_asset_artifact: &mut ProcessAssetArtifact,
  build_chunk_graph_artifact: &mut rspack_core::BuildChunkGraphArtifact,
) -> Result<()> {
  self.coordinator.idle().await?;
  Ok(())
}

impl Plugin for RscServerPlugin {
  fn name(&self) -> &'static str {
    "rspack.RscServerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));

    ctx.compiler_hooks.done.tap(done::new(self));
    ctx.compiler_hooks.failed.tap(failed::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));

    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}

impl RscServerPlugin {
  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut add_ssr_modules_list: Vec<InjectedSsrEntry> = Default::default();
    let mut created_ssr_dependencies_per_entry: FxHashMap<String, Vec<DependencyId>> =
      Default::default();
    let mut add_action_entry_list: Vec<InjectedActionEntry> = Default::default();
    let mut server_actions_per_entry: FxHashMap<String, FxHashMap<String, Vec<ActionIdNamePair>>> =
      Default::default();
    let mut created_action_ids: FxHashSet<String> = Default::default();
    let mut runtime_per_entry: FxHashMap<String, RuntimeSpec> = Default::default();

    for (entry_name, entry_data) in &compilation.entries {
      let runtime = get_entry_runtime(entry_name, &entry_data.options, &compilation.entries);
      runtime_per_entry.insert(entry_name.clone(), runtime.clone());

      let mut action_entry_imports: FxHashMap<String, Vec<ActionIdNamePair>> = Default::default();
      let mut client_entries_to_inject = Vec::new();

      let entry_dependency = &entry_data.dependencies[0];
      let component_info =
        collect_component_info_from_entry_denendency(compilation, &runtime, entry_dependency);
      for (dep, actions) in component_info.action_imports {
        action_entry_imports.insert(dep, actions);
      }
      if !component_info.client_component_imports.is_empty()
        || !component_info.css_imports.is_empty()
      {
        client_entries_to_inject.push(ClientEntry {
          entry_name: entry_name.clone(),
          runtime: runtime.clone(),
          client_imports: component_info.client_component_imports,
          css_imports: component_info.css_imports,
        });
      }

      {
        let mut plugin_state = PLUGIN_STATES.entry(compilation.compiler_id()).or_default();

        for client_entry_to_inject in client_entries_to_inject {
          let entry_name = client_entry_to_inject.entry_name.clone();
          let Some(injected) = self
            .inject_client_entry_and_ssr_modules(
              compilation,
              client_entry_to_inject,
              component_info.should_inject_ssr_modules,
              &mut plugin_state,
            )
            .await
          else {
            continue;
          };

          // Track all created SSR dependencies for each entry from the server layer.
          created_ssr_dependencies_per_entry
            .entry(entry_name)
            .or_default()
            .push(injected.dependency_id);

          add_ssr_modules_list.push(injected);
        }
      }

      if !action_entry_imports.is_empty() {
        server_actions_per_entry
          .entry(entry_name.clone())
          .or_default()
          .extend(action_entry_imports);
      }
    }

    for (name, action_entry_imports) in server_actions_per_entry {
      let runtime = runtime_per_entry.get(&name).cloned().unwrap_or_default();
      if let Some(injected) = self.inject_action_entry(
        compilation,
        ActionEntry {
          actions: action_entry_imports,
          entry_name: name.clone(),
          runtime,
          from_client: false,
        },
        &mut created_action_ids,
      ) {
        add_action_entry_list.push(injected);
      }
    }

    // Wait for action entries to be added.

    let included_dependencies: Vec<(DependencyId, RuntimeSpec)> = add_ssr_modules_list
      .iter()
      .map(|injected| (*injected.add_entry.0.id(), injected.runtime.clone()))
      .chain(
        add_action_entry_list
          .iter()
          .map(|injected| (*injected.add_entry.0.id(), injected.runtime.clone())),
      )
      .collect();
    let add_include_args: Vec<(BoxDependency, EntryOptions)> = add_ssr_modules_list
      .into_iter()
      .map(|injected_ssr_entry| injected_ssr_entry.add_entry)
      .chain(
        add_action_entry_list
          .into_iter()
          .map(|add_action_entry| add_action_entry.add_entry),
      )
      .collect();
    compilation.add_include(add_include_args).await?;
    for (dependency_id, runtime) in included_dependencies {
      let mg = compilation.get_module_graph_mut();
      let Some(module) = mg.get_module_by_dependency_id(&dependency_id) else {
        continue;
      };
      let info = mg.get_exports_info_data_mut(&module.identifier());
      info.set_used_in_unknown_way(Some(&runtime));
    }

    self
      .coordinator
      .complete_server_entries_compilation()
      .await?;

    self.coordinator.start_server_actions_compilation().await?;

    self
      .coordinator
      .complete_server_actions_compilation()
      .await?;

    let mut added_client_action_entry_list: Vec<InjectedActionEntry> = Vec::new();
    let plugin_state = PLUGIN_STATES
      .get(&compilation.compiler_id())
      .ok_or_else(|| {
        rspack_error::error!(
          "RscServerPlugin: Plugin state not found in finish_make hook for compiler {:#?}.",
          compilation.compiler_id()
        )
      })?;

    for (entry_name, action_entry_imports) in &plugin_state.client_actions_per_entry {
      // If an action method is already created in the server layer, we don't
      // need to create it again in the action layer.
      // This is to avoid duplicate action instances and make sure the module
      // state is shared.
      let mut remaining_client_imported_actions = false;
      let mut remaining_action_entry_imports: FxHashMap<String, Vec<ActionIdNamePair>> =
        Default::default();
      let runtime = runtime_per_entry
        .get(entry_name)
        .cloned()
        .unwrap_or_default();
      for (dependency, actions) in action_entry_imports {
        let mut remaining_actions: Vec<ActionIdNamePair> = Vec::new();
        for action in actions {
          if !created_action_ids.contains(&format!("{}@{}", entry_name, &action.0)) {
            remaining_actions.push(action.clone());
          }
        }
        if !remaining_actions.is_empty() {
          remaining_action_entry_imports.insert(dependency.clone(), remaining_actions);
          remaining_client_imported_actions = true;
        }
      }

      if remaining_client_imported_actions
        && let Some(injected) = self.inject_action_entry(
          compilation,
          ActionEntry {
            actions: remaining_action_entry_imports,
            entry_name: entry_name.clone(),
            runtime,
            from_client: true,
          },
          &mut created_action_ids,
        )
      {
        added_client_action_entry_list.push(injected);
      }
    }
    let included_dependencies: Vec<(DependencyId, RuntimeSpec)> = added_client_action_entry_list
      .iter()
      .map(|action_entry| (*action_entry.add_entry.0.id(), action_entry.runtime.clone()))
      .collect();
    let add_include_args: Vec<(BoxDependency, EntryOptions)> = added_client_action_entry_list
      .into_iter()
      .map(|action_entry| action_entry.add_entry)
      .collect();
    compilation.add_include(add_include_args).await?;
    for (dependency_id, runtime) in included_dependencies {
      let mg = compilation.get_module_graph_mut();
      let Some(m) = mg.get_module_by_dependency_id(&dependency_id) else {
        continue;
      };
      let info = mg.get_exports_info_data_mut(&m.identifier());
      info.set_used_in_unknown_way(Some(&runtime));
    }

    Ok(())
  }

  async fn inject_client_entry_and_ssr_modules(
    &self,
    compilation: &Compilation,
    client_entry: ClientEntry,
    should_inject_ssr_modules: bool,
    plugin_state: &mut PluginState,
  ) -> Option<InjectedSsrEntry> {
    let ClientEntry {
      entry_name,
      runtime,
      client_imports,
      css_imports,
    } = client_entry;

    let client_browser_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      let merged_css_imports = css_imports.values().flatten().collect::<FxHashSet<_>>();
      for request in merged_css_imports {
        #[allow(clippy::unwrap_used)]
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": []
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }

      plugin_state
        .entry_css_imports
        .entry(entry_name.clone())
        .or_default()
        .extend(css_imports.into_iter());

      for (request, ids) in &client_imports {
        #[allow(clippy::unwrap_used)]
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "false");
      format!(
        "{}?{}!",
        CLIENT_ENTRY_LOADER_IDENTIFIER,
        serializer.finish()
      )
    };

    let client_server_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      for (request, ids) in &client_imports {
        #[allow(clippy::unwrap_used)]
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "true");
      format!(
        "{}?{}!",
        CLIENT_ENTRY_LOADER_IDENTIFIER,
        serializer.finish()
      )
    };

    // Add for the client compilation
    // Inject the entry to the client compiler.
    plugin_state
      .injected_client_entries
      .insert(entry_name.clone(), client_browser_loader);

    if !should_inject_ssr_modules {
      return None;
    }

    let ssr_entry_dependency = EntryDependency::new(
      client_server_loader,
      compilation.options.context.clone(),
      Some(LAYERS_NAMES.server_side_rendering.to_string()),
      false,
    );
    let dependency_id = *(ssr_entry_dependency.id());
    Some(InjectedSsrEntry {
      runtime,
      add_entry: (
        Box::new(ssr_entry_dependency),
        EntryOptions {
          name: Some(entry_name),
          ..Default::default()
        },
      ),
      dependency_id,
    })
  }

  fn inject_action_entry(
    &self,
    compilation: &Compilation,
    action_entry: ActionEntry,
    created_action_ids: &mut FxHashSet<String>,
  ) -> Option<InjectedActionEntry> {
    let ActionEntry {
      actions,
      entry_name,
      runtime,
      from_client,
    } = action_entry;

    if actions.is_empty() {
      return None;
    }

    for actions_from_module in actions.values() {
      for (id, _) in actions_from_module {
        created_action_ids.insert(format!("{entry_name}@{id}"));
      }
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    #[allow(clippy::unwrap_used)]
    serializer.append_pair("actions", &serde_json::to_string(&actions).unwrap());
    serializer.append_pair("from-client", &from_client.to_string());
    let action_entry_loader = format!(
      "{}?{}!",
      ACTION_ENTRY_LOADER_IDENTIFIER,
      serializer.finish()
    );

    // Inject the entry to the server compiler
    let layer = LAYERS_NAMES.react_server_components.to_string();
    let action_entry_dep = EntryDependency::new(
      action_entry_loader,
      compilation.options.context.clone(),
      Some(layer.clone()),
      false,
    );

    Some(InjectedActionEntry {
      runtime,
      add_entry: (
        Box::new(action_entry_dep),
        EntryOptions {
          name: Some(entry_name),
          layer: Some(layer),
          ..Default::default()
        },
      ),
    })
  }
}

#[plugin_hook(CompilerDone for RscServerPlugin)]
async fn done(&self, compilation: &Compilation) -> Result<()> {
  if let Some(on_server_component_changes) = self.on_server_component_changes.as_ref() {
    let plugin_state = PLUGIN_STATES
      .get(&compilation.compiler_id())
      .ok_or_else(|| {
        rspack_error::error!(
          "RscServerPlugin: Plugin state not found in done hook for compiler {:#?}.",
          compilation.compiler_id()
        )
      })?;
    let changed_server_components_per_entry = plugin_state
      .changed_server_components_per_entry
      .iter()
      .filter(|(_, changes)| !changes.is_empty())
      .collect::<FxHashMap<_, _>>();
    if !changed_server_components_per_entry.is_empty() {
      (on_server_component_changes)().await?;
    }
  }
  Ok(())
}

#[plugin_hook(CompilerFailed for RscServerPlugin)]
async fn failed(&self, _compilation: &Compilation) -> Result<()> {
  self.coordinator.failed().await?;
  Ok(())
}
