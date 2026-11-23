#![allow(clippy::ref_option_ref)]

use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_collections::{Identifiable, IdentifierMap};
use rspack_core::{
  BoxDependency, ChunkByUkey, ChunkNamedIdArtifact, ChunkUkey, Compilation, CompilationChunkIds,
  CompilationParams, CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerDone,
  CompilerFailed, CompilerFinishMake, CompilerThisCompilation, DependencyId, DependencyType,
  EntryDependency, EntryOptions, Logger, Plugin, RuntimeGlobals, RuntimeModule, RuntimeSpec,
  get_entry_runtime,
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_intern::Atom;
use rspack_util::fx_hash::FxIndexMap;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  component_info::{
    ClientComponentImports, ClientComponentImportsByServerEntry, ComponentInfo,
    ImportMetaRscImporters, collect_component_info_from_entry_dependencies,
  },
  constants::{CSS_REGEX, LAYERS_NAMES},
  coordinator::Coordinator,
  hot_reloader::track_server_component_changes,
  loaders::action_entry_loader::ACTION_ENTRY_LOADER_IDENTIFIER,
  manifest_runtime_module::RscManifestRuntimeModule,
  plugin_state::{
    ActionIdNamePair, ClientModuleImport, ClientModulesByServerEntry, CssImportsByServerEntry,
    EntryState, PLUGIN_STATES, RootCssImports,
  },
  reference_manifest::{
    RscCssLinkProps, RscEntryManifest, RscManifest, build_server_consumer_module_map,
    build_server_manifest,
  },
  rsc_entry_dependency::RscEntryDependency,
  rsc_entry_module_factory::RscEntryModuleFactory,
};

#[derive(Debug, Default)]
struct ClientEntry {
  should_inject_ssr_modules: bool,
  client_imports: ClientComponentImports,
  async_client_component_imports: ClientComponentImports,
  root_client_imports: ClientComponentImports,
  client_imports_by_server_entry: ClientComponentImportsByServerEntry,
  css_imports_by_server_entry: CssImportsByServerEntry,
  root_css_imports: RootCssImports,
  import_meta_rsc_importers: ImportMetaRscImporters,
}

struct ServerEntryGraph {
  runtime: RuntimeSpec,
  client_action_entries: Vec<DependencyId>,
  created_action_ids: FxHashSet<Atom>,
  client_entry: ClientEntry,
  ssr_entry: Option<DependencyId>,
}

impl ClientEntry {
  fn from_component_info(info: ComponentInfo) -> Self {
    if info.client_component_imports.is_empty()
      && info.css_imports_by_server_entry.is_empty()
      && info.root_css_imports.is_empty()
      && info.import_meta_rsc_importers.is_empty()
    {
      return Self::default();
    }
    Self {
      should_inject_ssr_modules: info.should_inject_ssr_modules,
      client_imports: info.client_component_imports,
      async_client_component_imports: info.async_client_component_imports,
      root_client_imports: info.root_client_component_imports,
      client_imports_by_server_entry: info.client_component_imports_by_server_entry,
      css_imports_by_server_entry: info.css_imports_by_server_entry,
      root_css_imports: info.root_css_imports,
      import_meta_rsc_importers: info.import_meta_rsc_importers,
    }
  }

  fn has_same_dependencies(&self, other: &Self) -> bool {
    self.should_inject_ssr_modules == other.should_inject_ssr_modules
      && same_client_imports(&self.client_imports, &other.client_imports)
      && same_client_imports(
        &self.async_client_component_imports,
        &other.async_client_component_imports,
      )
      && same_client_imports(&self.root_client_imports, &other.root_client_imports)
      && self.client_imports_by_server_entry.len() == other.client_imports_by_server_entry.len()
      && self
        .client_imports_by_server_entry
        .iter()
        .zip(&other.client_imports_by_server_entry)
        .all(|((owner, imports), (other_owner, other_imports))| {
          owner == other_owner && same_client_imports(imports, other_imports)
        })
      && self.css_imports_by_server_entry.len() == other.css_imports_by_server_entry.len()
      && self
        .css_imports_by_server_entry
        .iter()
        .zip(&other.css_imports_by_server_entry)
        .all(|((owner, imports), (other_owner, other_imports))| {
          owner == other_owner && imports.iter().eq(other_imports)
        })
      && self.root_css_imports.iter().eq(&other.root_css_imports)
      && self.import_meta_rsc_importers == other.import_meta_rsc_importers
  }
}

fn same_client_imports(imports: &ClientComponentImports, other: &ClientComponentImports) -> bool {
  imports.len() == other.len()
    && imports
      .iter()
      .zip(other)
      .all(|((path, ids), (other_path, other_ids))| path == other_path && ids.iter().eq(other_ids))
}

struct ActionEntry {
  actions: FxHashMap<String, Vec<ActionIdNamePair>>,
  entry_name: Arc<str>,
  runtime: RuntimeSpec,
  from_client: bool,
}

#[derive(Debug)]
struct InjectedEntry {
  runtime: RuntimeSpec,
  add_entry: (BoxDependency, EntryOptions),
}

type OnServerComponentChanges = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub type OnManifest = Box<dyn Fn(String) -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct RscServerPluginOptions {
  pub coordinator: Arc<Coordinator>,
  pub css_link_props: RscCssLinkProps,
  pub on_server_component_changes: Option<OnServerComponentChanges>,
  pub on_manifest: Option<OnManifest>,
}

fn client_imports_to_modules(client_imports: &ClientComponentImports) -> Vec<ClientModuleImport> {
  client_imports
    .iter()
    .map(|(request, ids)| ClientModuleImport {
      request: request.clone(),
      ids: ids.iter().cloned().collect(),
    })
    .collect()
}

fn group_client_entries_by_owner(
  client_imports: &ClientComponentImports,
  async_client_component_imports: &ClientComponentImports,
  root_client_imports: &ClientComponentImports,
  client_imports_by_server_entry: &ClientComponentImportsByServerEntry,
) -> (
  Vec<ClientModuleImport>,
  Vec<ClientModuleImport>,
  ClientModulesByServerEntry,
) {
  let mut ungrouped_client_entries = Vec::new();
  let mut root_client_entries = Vec::new();
  let mut client_entries_by_server_entry: ClientModulesByServerEntry = Default::default();

  for (request, ids) in client_imports {
    let client_module = ClientModuleImport {
      request: request.clone(),
      ids: ids.iter().cloned().collect(),
    };

    // Preserve server-side dynamic import boundaries in the client compiler:
    // `RscEntryModule` emits ungrouped modules as one async block per module,
    // so a server `import("./Client")` keeps a matching client chunk instead
    // of being merged into the synchronous root/server-entry owner group.
    // The `client_module` above uses ids from `client_imports`, which keeps
    // the merged export set when the same module is reached by multiple paths.
    if async_client_component_imports.contains_key(request.as_str()) {
      ungrouped_client_entries.push(client_module);
      continue;
    }

    let mut owner_count = 0usize;
    let is_root_owned = root_client_imports.contains_key(request.as_str());
    let mut owned_server_entry = None;

    if is_root_owned {
      owner_count += 1;
    }

    for (server_entry, client_imports) in client_imports_by_server_entry {
      if client_imports.contains_key(request.as_str()) {
        owner_count += 1;
        if owned_server_entry.is_none() {
          owned_server_entry = Some(server_entry);
        }
      }
    }

    if owner_count == 1 {
      if is_root_owned {
        root_client_entries.push(client_module);
      } else if let Some(server_entry) = owned_server_entry {
        client_entries_by_server_entry
          .entry(server_entry.clone())
          .or_default()
          .push(client_module);
      } else {
        ungrouped_client_entries.push(client_module);
      }
    } else {
      ungrouped_client_entries.push(client_module);
    }
  }

  (
    ungrouped_client_entries,
    root_client_entries,
    client_entries_by_server_entry,
  )
}

#[plugin]
#[derive(Debug)]
pub struct RscServerPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
  #[debug(skip)]
  on_server_component_changes: Option<OnServerComponentChanges>,
  #[debug(skip)]
  on_manifest: Option<OnManifest>,
  css_link_props: RscCssLinkProps,
  prev_server_component_hashes: AtomicRefCell<IdentifierMap<u64>>,
}

impl RscServerPlugin {
  pub fn new(options: RscServerPluginOptions) -> Self {
    Self::new_inner(
      options.coordinator,
      options.on_server_component_changes,
      options.on_manifest,
      options.css_link_props,
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
  PLUGIN_STATES
    .entry(compilation.compiler_id())
    .or_default()
    .css_link_props = self.css_link_props.clone();

  self
    .coordinator
    .start_server_entries_compilation(compilation.compiler_id())
    .await?;

  Ok(())
}

#[plugin_hook(CompilerCompilation for RscServerPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::RscEntry, Arc::new(RscEntryModuleFactory));
  Ok(())
}

#[plugin_hook(CompilerFinishMake for RscServerPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.RscServerPlugin");

  {
    let mut plugin_state = PLUGIN_STATES.entry(compilation.compiler_id()).or_default();

    let start = logger.time("track server component changes");
    let mut prev_server_component_hashes = self.prev_server_component_hashes.borrow_mut();
    let changed_per_entry =
      track_server_component_changes(compilation, &mut prev_server_component_hashes);
    for (entry_name, set) in changed_per_entry {
      plugin_state
        .entries
        .entry(entry_name)
        .or_default()
        .changed_server_components = set;
    }
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

/// Compute server manifest and server_consumer_module_map once per entry. Stored in plugin_state for
/// RscManifestRuntimeModule and onManifest to avoid recomputing.
#[plugin_hook(CompilationChunkIds for RscServerPlugin, stage = -10000)]
async fn chunk_ids(
  &self,
  compilation: &Compilation,
  _chunk_by_ukey: &mut ChunkByUkey,
  _named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let mut plugin_state = PLUGIN_STATES.entry(compilation.compiler_id()).or_default();
  build_server_manifest(compilation, &mut plugin_state)?;
  for entry_state in plugin_state.entries.values_mut() {
    let map = build_server_consumer_module_map(compilation, &entry_state.client_modules);
    entry_state.server_consumer_module_map = Some(map);
  }
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

    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.done.tap(done::new(self));
    ctx.compiler_hooks.failed.tap(failed::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));

    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));

    Ok(())
  }
}

impl RscServerPlugin {
  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut entry_graphs: FxIndexMap<_, _> = compilation
      .entries
      .iter()
      .map(|(name, entry)| {
        (
          Arc::<str>::from(name.as_str()),
          ServerEntryGraph {
            runtime: get_entry_runtime(name, &entry.options, &compilation.entries),
            client_action_entries: Vec::new(),
            created_action_ids: FxHashSet::default(),
            client_entry: ClientEntry::default(),
            ssr_entry: None,
          },
        )
      })
      .collect();
    self
      .update_client_entries(compilation, &mut entry_graphs, true)
      .await?;
    self
      .coordinator
      .complete_server_entries_compilation()
      .await?;

    loop {
      self.coordinator.start_server_actions_compilation().await?;
      let mut client_actions: FxHashMap<_, _> = {
        let mut plugin_state = PLUGIN_STATES
          .get_mut(&compilation.compiler_id())
          .ok_or_else(|| {
            rspack_error::error!(
              "RscServerPlugin state not found while adding client-imported actions"
            )
          })?;
        plugin_state
          .entries
          .iter_mut()
          .map(|(name, state)| (name.clone(), std::mem::take(&mut state.client_actions)))
          .collect()
      };
      let mut includes = Vec::new();
      for (name, graph) in &mut entry_graphs {
        let Some(mut actions) = client_actions.remove(name) else {
          continue;
        };
        retain_new_actions(&mut actions, &graph.created_action_ids);
        if let Some(injected) = self.inject_action_entry(
          compilation,
          ActionEntry {
            actions,
            entry_name: name.clone(),
            runtime: graph.runtime.clone(),
            from_client: true,
          },
          &mut graph.created_action_ids,
        ) {
          graph.client_action_entries.push(*injected.add_entry.0.id());
          includes.push(injected);
        }
      }

      // Only a previously unseen action can grow the server graph in this loop.
      if !includes.is_empty() {
        build_injected_entries(compilation, includes).await?;
        if self
          .update_client_entries(compilation, &mut entry_graphs, false)
          .await?
        {
          self.coordinator.publish_updated_client_entries().await?;
          continue;
        }
      }
      self
        .coordinator
        .complete_server_actions_compilation()
        .await?;
      return self.coordinator.wait_for_client_manifest().await;
    }
  }

  async fn update_client_entries(
    &self,
    compilation: &mut Compilation,
    entry_graphs: &mut FxIndexMap<Arc<str>, ServerEntryGraph>,
    initial: bool,
  ) -> Result<bool> {
    let mut includes = Vec::new();
    let mut action_includes = Vec::new();
    let mut updated_client_entries = Vec::new();
    let updated = {
      let mut plugin_state = PLUGIN_STATES.entry(compilation.compiler_id()).or_default();
      for (name, graph) in entry_graphs {
        if !initial && graph.client_action_entries.is_empty() {
          continue;
        }
        let Some(entry) = compilation.entries.get(name.as_ref()) else {
          continue;
        };
        let module_graph = compilation.get_module_graph();
        // Start from the action modules themselves. The generated action entry
        // has no resource and the component collector skips it.
        let action_dependencies = graph
          .client_action_entries
          .iter()
          .filter_map(|id| module_graph.get_resolved_module(id))
          .flat_map(|module| module_graph.get_outgoing_deps_in_order(module));
        let mut info = collect_component_info_from_entry_dependencies(
          compilation,
          &graph.runtime,
          entry.dependencies.iter().chain(action_dependencies),
        );
        let mut server_actions = std::mem::take(&mut info.action_imports)
          .into_iter()
          .collect();
        if !initial {
          retain_new_actions(&mut server_actions, &graph.created_action_ids);
        }
        if let Some(injected) = self.inject_action_entry(
          compilation,
          ActionEntry {
            actions: server_actions,
            entry_name: name.clone(),
            runtime: graph.runtime.clone(),
            from_client: false,
          },
          &mut graph.created_action_ids,
        ) {
          action_includes.push(injected);
        }

        let client_entry = ClientEntry::from_component_info(info);
        if client_entry.has_same_dependencies(&graph.client_entry) {
          continue;
        }
        if let Some(previous) = graph.ssr_entry.take()
          && let Some(entry) = compilation.entries.get_mut(name.as_ref())
        {
          entry.include_dependencies.retain(|id| *id != previous);
        }
        let entry_state = plugin_state.entries.entry(name.clone()).or_default();
        if let Some(injected) =
          self.inject_client_entry_and_ssr_modules(name, &graph.runtime, &client_entry, entry_state)
        {
          graph.ssr_entry = Some(*injected.add_entry.0.id());
          includes.push(injected);
        }
        graph.client_entry = client_entry;
        updated_client_entries.push(name.clone());
      }
      let updated = !updated_client_entries.is_empty();
      plugin_state.updated_client_entries = updated_client_entries;
      updated
    };

    includes.extend(action_includes);
    build_injected_entries(compilation, includes).await?;
    Ok(updated)
  }

  fn inject_client_entry_and_ssr_modules(
    &self,
    entry_name: &Arc<str>,
    runtime: &RuntimeSpec,
    client_entry: &ClientEntry,
    entry_state: &mut EntryState,
  ) -> Option<InjectedEntry> {
    let ClientEntry {
      should_inject_ssr_modules,
      client_imports,
      async_client_component_imports,
      root_client_imports,
      client_imports_by_server_entry,
      css_imports_by_server_entry,
      root_css_imports,
      import_meta_rsc_importers,
    } = client_entry;

    let client_entries = client_imports_to_modules(client_imports);
    let (ungrouped_client_entries, root_client_entries, client_entries_by_server_entry) =
      group_client_entries_by_owner(
        client_imports,
        async_client_component_imports,
        root_client_imports,
        client_imports_by_server_entry,
      );

    {
      entry_state.server_entries.clear();
      for (server_entry, css_imports) in css_imports_by_server_entry {
        entry_state
          .server_entries
          .entry(server_entry.clone())
          .or_default()
          .css_imports
          .extend(css_imports.iter().cloned());
      }
      for (server_entry, importers) in import_meta_rsc_importers {
        entry_state
          .server_entries
          .entry(server_entry.clone())
          .or_default()
          .import_meta_rsc_importers
          .extend(importers.iter().cloned());
      }
      entry_state.root_css_imports.clone_from(root_css_imports);

      // These grouped client entries are only used to shape client compiler
      // async blocks. Do not create a `server_entries[server_entry]` record
      // from `client_entries_by_server_entry` alone: a server entry that only
      // owns client components has no server CSS block to record here.
      //
      // Client component CSS is collected from the client module chunks and
      // stored on `clientManifest[*].cssFiles` when that client module is
      // recorded. `server_entries` is intentionally reserved for server CSS
      // imports and `import.meta.rspackRsc.loadCss()` data, which are the only
      // sources that should populate `entryCssFiles`.
      entry_state.injected_client_entries = client_entries.clone();
      entry_state.ungrouped_client_entries = ungrouped_client_entries;
      entry_state.root_client_entries = root_client_entries;
      entry_state.client_entries_by_server_entry = client_entries_by_server_entry;
    }

    // Add for the client compilation
    // Inject the entry to the client compiler.
    if !*should_inject_ssr_modules {
      return None;
    }

    // For SSR, filter out CSS (server does not need CSS in the eager entry).
    let client_entries_for_ssr: Vec<ClientModuleImport> = client_entries
      .iter()
      .filter(|m| !CSS_REGEX.is_match(&m.request))
      .map(|m| ClientModuleImport {
        request: m.request.clone(),
        ids: m.ids.clone(),
      })
      .collect();
    let ssr_entry_dependency = RscEntryDependency::new(
      entry_name.clone(),
      client_entries_for_ssr,
      Default::default(),
      Default::default(),
      Default::default(),
      true,
    );
    Some(InjectedEntry {
      runtime: runtime.clone(),
      add_entry: (
        BoxDependency::new(ssr_entry_dependency),
        EntryOptions {
          name: Some(entry_name.to_string()),
          layer: Some(LAYERS_NAMES.server_side_rendering.to_string()),
          ..Default::default()
        },
      ),
    })
  }

  fn inject_action_entry(
    &self,
    compilation: &Compilation,
    action_entry: ActionEntry,
    created_action_ids: &mut FxHashSet<Atom>,
  ) -> Option<InjectedEntry> {
    let ActionEntry {
      actions,
      entry_name,
      runtime,
      from_client,
    } = action_entry;

    if actions.is_empty() {
      return None;
    }
    created_action_ids.extend(actions.values().flatten().map(|(id, _)| id.clone()));

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    #[allow(clippy::unwrap_used)]
    serializer.append_pair("actions", &simd_json::to_string(&actions).unwrap());
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

    Some(InjectedEntry {
      runtime,
      add_entry: (
        BoxDependency::new(action_entry_dep),
        EntryOptions {
          name: Some(entry_name.to_string()),
          layer: Some(layer),
          ..Default::default()
        },
      ),
    })
  }
}

fn retain_new_actions(
  actions: &mut FxHashMap<String, Vec<ActionIdNamePair>>,
  created_action_ids: &FxHashSet<Atom>,
) {
  actions.retain(|_, actions| {
    actions.retain(|(id, _)| !created_action_ids.contains(id));
    !actions.is_empty()
  });
}

async fn build_injected_entries(
  compilation: &mut Compilation,
  entries: Vec<InjectedEntry>,
) -> Result<()> {
  if entries.is_empty() {
    return Ok(());
  }
  let mut dependencies = Vec::with_capacity(entries.len());
  let mut includes = Vec::with_capacity(entries.len());
  for entry in entries {
    dependencies.push((*entry.add_entry.0.id(), entry.runtime));
    includes.push(entry.add_entry);
  }
  compilation.add_include(includes).await?;
  for (id, runtime) in dependencies {
    let Some(module) = compilation
      .get_module_graph()
      .get_module_by_dependency_id(&id)
      .map(|module| module.identifier())
    else {
      continue;
    };
    compilation
      .exports_info_artifact
      .get_exports_info_data_mut(&module)
      .set_used_in_unknown_way(Some(&runtime));
  }
  Ok(())
}

#[plugin_hook(CompilerDone for RscServerPlugin)]
async fn done(&self, compilation: &Compilation) -> Result<()> {
  if let Some(on_manifest) = self.on_manifest.as_ref() {
    let json = {
      let plugin_state = PLUGIN_STATES
        .get(&compilation.compiler_id())
        .ok_or_else(|| {
          rspack_error::error!(
            "RscServerPlugin: Plugin state not found in done hook for compiler {:#?}.",
            compilation.compiler_id()
          )
        })?;
      let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
        rspack_error::error!(
          "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
        )
      })?;
      let empty_consumer_map: FxHashMap<String, crate::reference_manifest::ManifestNode> =
        FxHashMap::default();
      let mut full_manifest: RscManifest<'_> = FxHashMap::default();
      for (name, state) in &plugin_state.entries {
        full_manifest.insert(
          name.clone(),
          RscEntryManifest {
            server_manifest: &state.server_actions,
            client_manifest: &state.client_modules,
            server_consumer_module_map: state
              .server_consumer_module_map
              .as_ref()
              .or(Some(&empty_consumer_map)),
            module_loading,
            server_entries: &state.server_entries,
            bootstrap_scripts: &state.bootstrap_scripts,
            css_link_props: &plugin_state.css_link_props,
          },
        );
      }
      simd_json::to_string(&full_manifest).to_rspack_result()?
    };
    (on_manifest)(json).await?;
  }

  if let Some(on_server_component_changes) = self.on_server_component_changes.as_ref() {
    let has_changed_server_components = {
      let plugin_state = PLUGIN_STATES
        .get(&compilation.compiler_id())
        .ok_or_else(|| {
          rspack_error::error!(
            "RscServerPlugin: Plugin state not found in done hook for compiler {:#?}.",
            compilation.compiler_id()
          )
        })?;
      plugin_state
        .entries
        .values()
        .any(|e| !e.changed_server_components.is_empty())
    };
    if has_changed_server_components {
      (on_server_component_changes)().await?;
    }
  }

  self.coordinator.idle().await?;
  Ok(())
}

#[plugin_hook(CompilerFailed for RscServerPlugin)]
async fn failed(&self, _compilation: &Compilation, error: &rspack_error::Error) -> Result<()> {
  self.coordinator.failed(error).await;
  Ok(())
}
