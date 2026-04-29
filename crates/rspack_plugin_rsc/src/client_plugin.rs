use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilationAfterProcessAssets,
  CompilationParams, CompilerCompilation, CompilerFailed, CompilerId, CompilerMake,
  CrossOriginLoading, DependenciesBlock, Dependency, DependencyId, DependencyType, EntryDependency,
  Logger, ModuleGraph, ModuleId, ModuleIdentifier, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  Coordinator,
  plugin_state::{ActionIdNamePair, EntryState, PLUGIN_STATES, PluginState},
  reference_manifest::{CrossOriginMode, ManifestExport, ModuleLoading},
  rsc_entry_dependency::RscEntryDependency,
  rsc_entry_module::RscEntryModule,
  rsc_entry_module_factory::RscEntryModuleFactory,
  utils::{encode_uri_path, get_module_resource, is_css_mod},
};

pub struct RscClientPluginOptions {
  pub coordinator: Arc<Coordinator>,
}

#[plugin]
#[derive(Debug)]
pub struct RscClientPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
  server_compiler_id: AtomicRefCell<Option<CompilerId>>,
  client_entries_per_entry: AtomicRefCell<FxHashMap<Arc<str>, FxHashSet<DependencyId>>>,
}

fn extend_required_chunks(
  chunk_group: &ChunkGroup,
  compilation: &Compilation,
  required_chunks: &mut Vec<String>,
) {
  for chunk_ukey in &chunk_group.chunks {
    let Some(chunk) = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(chunk_ukey)
    else {
      continue;
    };
    let Some(chunk_id) = chunk.id() else {
      continue;
    };
    for file in chunk.files().iter().filter(|f| f.ends_with(".js")) {
      if let Some(asset) = compilation.assets().get(file) {
        let asset_info = asset.get_info();
        if asset_info.hot_module_replacement.unwrap_or(false)
          || asset_info.development.unwrap_or(false)
        {
          continue;
        }
      };
      required_chunks.push(chunk_id.to_string());
      // We encode the file as a URI because our server (and many other services such as S3)
      // expect to receive reserved characters such as `[` and `]` as encoded. This was
      // previously done for dynamic chunks by patching the Rspack runtime but we want
      // these filenames to be managed by React's Flight runtime instead and so we need
      // to implement any special handling of the file name here.
      required_chunks.push(encode_uri_path(file));
    }
  }
}

fn prefixed_asset_path(prefix: &str, file: &str) -> String {
  format!("{prefix}{}", encode_uri_path(file))
}

#[allow(clippy::too_many_arguments)]
fn record_module(
  module_loading: &ModuleLoading,
  module_id: &ModuleId,
  module_identifier: &ModuleIdentifier,
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation,
  required_chunks: &[String],
  entry_state: &mut EntryState,
) {
  let Some(module) = compilation.module_by_identifier(module_identifier) else {
    return;
  };

  let resource = get_module_resource(module.as_ref());
  if resource.is_empty() {
    return;
  }

  if is_css_mod(module.as_ref()) {
    let mut matched_server_entries = Vec::new();
    for (server_entry, server_entry_state) in &entry_state.server_entries {
      if server_entry_state.css_imports.contains(resource.as_ref()) {
        matched_server_entries.push(server_entry.clone());
      }
    }
    if matched_server_entries.is_empty() {
      return;
    }

    let Some(chunk) = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(chunk_ukey)
    else {
      return;
    };

    let prefix = &module_loading.prefix;
    let css_files: Vec<String> = chunk
      .files()
      .iter()
      .filter(|file| file.ends_with(".css"))
      .map(|file| prefixed_asset_path(prefix, file))
      .collect();
    if css_files.is_empty() {
      return;
    }

    for server_entry in matched_server_entries {
      entry_state
        .server_entries
        .entry(server_entry)
        .or_default()
        .css_files
        .extend(css_files.clone());
    }

    return;
  }

  let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, module_identifier);
  let css_files: Vec<String> = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
    .map(|chunk| {
      let prefix = &module_loading.prefix;
      chunk
        .files()
        .iter()
        .filter(|file| file.ends_with(".css"))
        .map(|file| prefixed_asset_path(prefix, file))
        .collect()
    })
    .unwrap_or_default();
  entry_state.client_modules.insert(
    resource.to_string(),
    ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: required_chunks.to_vec(),
      css_files,
      r#async: Some(is_async),
    },
  );
}

#[allow(clippy::too_many_arguments)]
fn record_chunk_group(
  module_loading: &ModuleLoading,
  client_entry_modules: &IdentifierSet,
  chunk_group: &ChunkGroup,
  compilation: &Compilation,
  required_chunks: &mut Vec<String>,
  checked_chunk_groups: &mut FxHashSet<ChunkGroupUkey>,
  checked_chunks: &mut FxHashSet<ChunkUkey>,
  entry_state: &mut EntryState,
) {
  // Ensure recursion is stopped if we've already checked this chunk group.
  if checked_chunk_groups.contains(&chunk_group.ukey) {
    return;
  }
  checked_chunk_groups.insert(chunk_group.ukey);

  let module_graph = compilation.get_module_graph();

  // Only apply following logic to client module requests from client entry,
  // or if the module is marked as client module. That's because other
  // client modules don't need to be in the manifest at all as they're
  // never be referenced by the server/client boundary.
  // This saves a lot of bytes in the manifest.
  for chunk_ukey in &chunk_group.chunks {
    // Ensure recursion is stopped if we've already checked this chunk.
    if checked_chunks.contains(chunk_ukey) {
      continue;
    }
    checked_chunks.insert(*chunk_ukey);

    let chunk_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(chunk_ukey);
    for module_identifier in chunk_modules {
      if !client_entry_modules.contains(module_identifier) {
        continue;
      }
      let Some(module_id) =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
      else {
        continue;
      };
      let Some(module) = module_graph.module_by_identifier(module_identifier) else {
        continue;
      };

      if let Some(concatenated_module) = module.as_concatenated_module() {
        record_module(
          module_loading,
          module_id,
          &concatenated_module.get_root(),
          chunk_ukey,
          compilation,
          required_chunks,
          entry_state,
        );
      } else {
        record_module(
          module_loading,
          module_id,
          module_identifier,
          chunk_ukey,
          compilation,
          required_chunks,
          entry_state,
        );
      }
    }
  }

  // Walk through all children chunk groups too.
  for child_ukey in chunk_group.children_iterable() {
    let Some(child) = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(child_ukey)
    else {
      continue;
    };
    let start_len = required_chunks.len();
    extend_required_chunks(child, compilation, required_chunks);
    record_chunk_group(
      module_loading,
      client_entry_modules,
      child,
      compilation,
      required_chunks,
      checked_chunk_groups,
      checked_chunks,
      entry_state,
    );
    required_chunks.truncate(start_len);
  }
}

fn collect_bootstrap_scripts(
  compilation: &Compilation,
  plugin_state: &mut PluginState,
) -> Result<()> {
  for (entry_name, chunk_group_ukey) in &compilation.build_chunk_graph_artifact.entrypoints {
    let Some(entry_state) = plugin_state.entries.get_mut(entry_name.as_str()) else {
      continue;
    };

    let Some(chunk_group) = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(chunk_group_ukey)
    else {
      continue;
    };

    let prefix = &plugin_state
      .module_loading
      .as_ref()
      .expect("module_loading should be initialized in traverse_modules before recording modules")
      .prefix;

    let bootstrap_scripts = chunk_group
      .get_files(&compilation.build_chunk_graph_artifact.chunk_by_ukey)
      .into_iter()
      .filter(|chunk_file| chunk_file.ends_with(".js"))
      .filter(|chunk_file| {
        let Some(asset) = compilation.assets().get(chunk_file) else {
          return true;
        };
        // Prevent hot-module files from being included
        let asset_info = asset.get_info();
        !(asset_info.hot_module_replacement.unwrap_or(false)
          || asset_info.development.unwrap_or(false))
      })
      .map(|file| prefixed_asset_path(prefix, &file))
      .collect::<FxIndexSet<String>>();

    entry_state.bootstrap_scripts = bootstrap_scripts;
  }
  Ok(())
}

fn collect_actions(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  collected_actions: &mut FxHashMap<String, Vec<ActionIdNamePair>>,
  visited_modules: &mut IdentifierSet,
) {
  let module = match module_graph.module_by_identifier(module_identifier) {
    Some(m) => m,
    None => return,
  };

  let module_resource = get_module_resource(module.as_ref());
  if module_resource.is_empty() {
    return;
  }

  if visited_modules.contains(module_identifier) {
    return;
  }
  visited_modules.insert(*module_identifier);

  if let Some(action_ids) = module.build_info().rsc.as_ref().map(|rsc| &rsc.action_ids) {
    let pairs = action_ids
      .into_iter()
      .map(|(id, exported_name)| (id.clone(), exported_name.clone()))
      .collect::<Vec<_>>();

    collected_actions.insert(module_resource.to_string(), pairs);
  }

  // Collect used exported actions transversely.
  for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
    let Some(resolved_module) = module_graph.get_resolved_module(dependency_id) else {
      continue;
    };
    collect_actions(
      module_graph,
      resolved_module,
      collected_actions,
      visited_modules,
    );
  }
}

fn collect_client_actions_from_dependencies(
  compilation: &Compilation,
  entry_dependencies: &FxHashSet<DependencyId>,
) -> FxHashMap<String, Vec<ActionIdNamePair>> {
  // action file path -> action names
  let mut collected_actions: FxHashMap<String, Vec<ActionIdNamePair>> = Default::default();

  // Keep track of checked modules to avoid infinite loops with recursive imports.
  let mut visited_modules: IdentifierSet = Default::default();

  let module_graph = compilation.get_module_graph();
  for entry_dependency_id in entry_dependencies {
    let Some(entry_module_identifier) = module_graph.get_resolved_module(entry_dependency_id)
    else {
      continue;
    };
    for dependency_id in module_graph.get_outgoing_deps_in_order(entry_module_identifier) {
      let Some(module_identifier) = module_graph.get_resolved_module(dependency_id) else {
        continue;
      };
      collect_actions(
        module_graph,
        module_identifier,
        &mut collected_actions,
        &mut visited_modules,
      );
    }
  }

  collected_actions
}

impl RscClientPlugin {
  pub fn new(options: RscClientPluginOptions) -> Self {
    Self::new_inner(options.coordinator, Default::default(), Default::default())
  }

  fn traverse_modules(
    &self,
    compilation: &Compilation,
    plugin_state: &mut PluginState,
  ) -> Result<()> {
    let public_path = &compilation.options.output.public_path;
    let configured_cross_origin_loading = &compilation.options.output.cross_origin_loading;

    let prefix = match public_path {
      rspack_core::PublicPath::Filename(filename) => match filename.template() {
        Some(template) => template.to_string(),
        None => {
          return Err(rspack_error::error!(
            "Expected Rspack publicPath to be a string when using React Server Components."
          ));
        }
      },
      rspack_core::PublicPath::Auto => "/".to_string(),
    };

    let cross_origin: Option<CrossOriginMode> = match configured_cross_origin_loading {
      CrossOriginLoading::Enable(value) => {
        if value == "use-credentials" {
          Some(CrossOriginMode::UseCredentials)
        } else {
          Some(CrossOriginMode::Anonymous)
        }
      }
      _ => None,
    };

    plugin_state.module_loading = Some(ModuleLoading {
      prefix,
      cross_origin,
    });

    let mut client_entry_modules: IdentifierSet = Default::default();
    let module_graph = compilation.get_module_graph();
    for entry_data in compilation.entries.values() {
      for dependency_id in &entry_data.include_dependencies {
        let Some(module_identifier) =
          module_graph.module_identifier_by_dependency_id(dependency_id)
        else {
          continue;
        };
        let Some(module) = module_graph.module_by_identifier(module_identifier) else {
          continue;
        };

        // Check if the module is a RscEntryModule (our custom virtual module)
        let is_rsc_entry_module = module.downcast_ref::<RscEntryModule>().is_some();
        if !is_rsc_entry_module {
          continue;
        }
        // Traverse the blocks of the RscEntryModule to find the actual client modules
        for block_id in module.get_blocks() {
          let Some(block) = module_graph.block_by_id(block_id) else {
            continue;
          };
          for dep_id in block.get_dependencies() {
            if let Some(conn) = module_graph.connection_by_dependency_id(dep_id) {
              client_entry_modules.insert(*conn.module_identifier());
            }
          }
        }
      }
    }

    let mut required_chunks: Vec<String> = Default::default();
    let mut checked_chunk_groups: FxHashSet<ChunkGroupUkey> = Default::default();
    let mut checked_chunks: FxHashSet<ChunkUkey> = Default::default();

    for (entry_name, entrypoint_ukey) in &compilation.build_chunk_graph_artifact.entrypoints {
      let module_loading = plugin_state
        .module_loading
        .as_ref()
        .expect("module_loading should be initialized before recording modules");
      let Some(entry_state) = plugin_state.entries.get_mut(entry_name.as_str()) else {
        continue;
      };

      let Some(entrypoint) = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .get(entrypoint_ukey)
      else {
        continue;
      };

      required_chunks.clear();
      checked_chunk_groups.clear();
      checked_chunks.clear();

      record_chunk_group(
        module_loading,
        &client_entry_modules,
        entrypoint,
        compilation,
        &mut required_chunks,
        &mut checked_chunk_groups,
        &mut checked_chunks,
        entry_state,
      );
    }

    Ok(())
  }
}

impl Plugin for RscClientPlugin {
  fn name(&self) -> &'static str {
    "RscClientPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx.compiler_hooks.make.tap(make::new(self));

    ctx.compiler_hooks.failed.tap(failed::new(self));

    ctx
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for RscClientPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::RscEntry, Arc::new(RscEntryModuleFactory));
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  compilation.set_dependency_factory(
    DependencyType::RscClientReference,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

// Execution must occur after EntryPlugin to ensure base entries are established
// before injecting client component entries. Stage 100 ensures proper ordering.
#[plugin_hook(CompilerMake for RscClientPlugin, stage = 100)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  self.coordinator.start_client_entries_compilation().await?;

  let server_compiler_id = self.coordinator.get_server_compiler_id().await?;
  *self.server_compiler_id.borrow_mut() = Some(server_compiler_id);

  let plugin_state = PLUGIN_STATES.get(&server_compiler_id).ok_or_else(|| {
    rspack_error::error!(
      "RscClientPlugin: Plugin state not found in make hook for compiler {:#?}.",
      compilation.compiler_id()
    )
  })?;

  for (entry_name, entry_state) in &plugin_state.entries {
    let client_modules = &entry_state.injected_client_entries;
    if compilation.entries.get(entry_name.as_ref()).is_none() {
      compilation.push_diagnostic(Diagnostic::error(
        "RSC Client Entry Mismatch".to_string(),
        format!(
          "Entry '{}' not found in the client compiler. Failed to inject the following client modules: {}",
          entry_name,
          client_modules
            .iter()
            .map(|m| m.request.as_str())
            .collect::<Vec<_>>()
            .join(", ")
        ),
      ));
      continue;
    }

    let mut include_dependencies = Vec::new();
    if !client_modules.is_empty() || entry_state.has_css_imports_by_server_entry() {
      let dependency = Box::new(RscEntryDependency::new(
        entry_name.clone(),
        client_modules.clone(),
        entry_state.css_imports_by_server_entry(),
        false,
      ));
      self
        .client_entries_per_entry
        .borrow_mut()
        .entry(entry_name.clone())
        .or_default()
        .insert(*dependency.id());
      include_dependencies.push(*dependency.id());
      compilation
        .get_module_graph_mut()
        .add_dependency(dependency);
    }

    let mut entry_dependencies = Vec::new();
    for request in &entry_state.root_css_imports {
      let dependency = Box::new(EntryDependency::new(
        request.clone(),
        compilation.options.context.clone(),
        None,
        false,
      ));
      entry_dependencies.push(*dependency.id());
      compilation
        .get_module_graph_mut()
        .add_dependency(dependency);
    }

    #[allow(clippy::unwrap_used)]
    let entry_data = compilation.entries.get_mut(entry_name.as_ref()).unwrap();
    entry_data.dependencies.append(&mut entry_dependencies);
    entry_data
      .include_dependencies
      .append(&mut include_dependencies);
  }

  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for RscClientPlugin)]
async fn after_process_assets(
  &self,
  compilation: &Compilation,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.RscClientPlugin");

  let server_compiler_id = self.coordinator.get_server_compiler_id().await?;

  {
    let Some(mut plugin_state) = PLUGIN_STATES.get_mut(&server_compiler_id) else {
      return Err(rspack_error::error!(
        "Failed to find plugin state for server compiler (ID: {}). \
       The server compiler may not have properly collected client entry information, \
       or the compiler has not been initialized yet.",
        server_compiler_id.as_u32()
      ));
    };

    let start = logger.time("create client reference manifest");
    self.traverse_modules(compilation, &mut plugin_state)?;
    logger.time_end(start);

    let start = logger.time("record bootstrap scripts");
    collect_bootstrap_scripts(compilation, &mut plugin_state)?;
    logger.time_end(start);

    for (entry_name, client_entries) in self.client_entries_per_entry.borrow().iter() {
      let client_actions = collect_client_actions_from_dependencies(compilation, client_entries);
      plugin_state
        .entries
        .entry(entry_name.clone())
        .or_default()
        .client_actions = client_actions;
    }
  }

  self
    .coordinator
    .complete_client_entries_compilation()
    .await?;

  Ok(())
}

#[plugin_hook(CompilerFailed for RscClientPlugin)]
async fn failed(&self, _compilation: &Compilation) -> Result<()> {
  self.coordinator.failed().await?;
  Ok(())
}
