use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilationAfterProcessAssets,
  CompilerFailed, CompilerId, CompilerMake, CrossOriginLoading, Dependency, DependencyId,
  EntryDependency, Logger, ModuleGraph, ModuleId, ModuleIdentifier, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  Coordinator,
  loaders::client_entry_loader::{
    CLIENT_ENTRY_LOADER_IDENTIFIER, ParsedClientEntries, parse_client_entries,
  },
  plugin_state::{ActionIdNamePair, PLUGIN_STATES, PluginState},
  reference_manifest::{CrossOriginMode, ManifestExport, ModuleLoading},
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
  client_entries_per_entry: AtomicRefCell<FxHashMap<String, FxHashSet<DependencyId>>>,
}

fn extend_required_chunks(
  chunk_group: &ChunkGroup,
  compilation: &Compilation,
  required_chunks: &mut Vec<String>,
) {
  for chunk_ukey in &chunk_group.chunks {
    let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) else {
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

#[allow(clippy::too_many_arguments)]
fn record_module(
  entry_name: &str,
  module_id: &ModuleId,
  module_identifier: &ModuleIdentifier,
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation,
  required_chunks: &[String],
  plugin_state: &mut PluginState,
) {
  let Some(module) = compilation.module_by_identifier(module_identifier) else {
    return;
  };

  let resource = get_module_resource(module.as_ref());
  if resource.is_empty() {
    return;
  }

  if is_css_mod(module.as_ref()) {
    let (Some(chunk), Some(entry_css_imports)) = (
      compilation.chunk_by_ukey.get(chunk_ukey),
      plugin_state.entry_css_imports.get(entry_name),
    ) else {
      return;
    };

    let prefix = &plugin_state
      .module_loading
      .as_ref()
      .expect("module_loading should be initialized in traverse_modules before recording modules")
      .prefix;
    let css_files: Vec<String> = chunk
      .files()
      .iter()
      .filter(|file| file.ends_with(".css"))
      .map(|file| format!("{prefix}{file}"))
      .collect();
    if css_files.is_empty() {
      return;
    }

    let entry_css_files = plugin_state
      .entry_css_files
      .entry(entry_name.to_string())
      .or_default();

    for (server_entry, imports) in entry_css_imports {
      if imports.get(resource.as_ref()).is_some() {
        entry_css_files
          .entry(server_entry.clone())
          .or_default()
          .extend(css_files.iter().cloned());
      }
    }
    return;
  }

  let is_async = ModuleGraph::is_async(
    &compilation.async_modules_artifact.borrow(),
    module_identifier,
  );
  plugin_state.client_modules.insert(
    resource.to_string(),
    ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: required_chunks.to_vec(),
      r#async: Some(is_async),
    },
  );
}

#[allow(clippy::too_many_arguments)]
fn record_chunk_group(
  entry_name: &str,
  client_entry_modules: &FxHashSet<ModuleIdentifier>,
  chunk_group: &ChunkGroup,
  compilation: &Compilation,
  required_chunks: &mut Vec<String>,
  checked_chunk_groups: &mut FxHashSet<ChunkGroupUkey>,
  checked_chunks: &mut FxHashSet<ChunkUkey>,
  plugin_state: &mut PluginState,
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
          entry_name,
          module_id,
          &concatenated_module.get_root(),
          chunk_ukey,
          compilation,
          required_chunks,
          plugin_state,
        );
      } else {
        record_module(
          entry_name,
          module_id,
          module_identifier,
          chunk_ukey,
          compilation,
          required_chunks,
          plugin_state,
        );
      }
    }
  }

  // Walk through all children chunk groups too.
  for child_ukey in chunk_group.children_iterable() {
    let Some(child) = compilation.chunk_group_by_ukey.get(child_ukey) else {
      continue;
    };
    let start_len = required_chunks.len();
    extend_required_chunks(child, compilation, required_chunks);
    record_chunk_group(
      entry_name,
      client_entry_modules,
      child,
      compilation,
      required_chunks,
      checked_chunk_groups,
      checked_chunks,
      plugin_state,
    );
    required_chunks.truncate(start_len);
  }
}

async fn collect_entry_js_files(
  compilation: &Compilation,
  plugin_state: &mut PluginState,
) -> Result<()> {
  for (entry_name, chunk_group_ukey) in &compilation.entrypoints {
    let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group_ukey) else {
      continue;
    };
    let entry_js_files = plugin_state
      .entry_js_files
      .entry(entry_name.clone())
      .or_default();
    let prefix = &plugin_state
      .module_loading
      .as_ref()
      .expect("module_loading should be initialized in traverse_modules before recording modules")
      .prefix;

    *entry_js_files = chunk_group
      .get_files(&compilation.chunk_by_ukey)
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
      .map(|file| format!("{prefix}{file}"))
      .collect::<FxIndexSet<String>>();
  }
  Ok(())
}

fn collect_actions(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  collected_actions: &mut FxHashMap<String, Vec<ActionIdNamePair>>,
  visited_modules: &mut FxHashSet<ModuleIdentifier>,
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
  let mut visited_modules: FxHashSet<Identifier> = Default::default();

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

  async fn traverse_modules(
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

    let mut client_entry_modules: FxHashSet<ModuleIdentifier> = Default::default();
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

        let is_client_loader = module
          .as_normal_module()
          .is_some_and(|m| m.user_request().starts_with(CLIENT_ENTRY_LOADER_IDENTIFIER));
        if !is_client_loader {
          continue;
        }
        for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
          if let Some(conn) = module_graph.connection_by_dependency_id(dependency_id) {
            client_entry_modules.insert(*conn.module_identifier());
          }
        }
      }
    }

    let mut required_chunks: Vec<String> = Default::default();
    let mut checked_chunk_groups: FxHashSet<ChunkGroupUkey> = Default::default();
    let mut checked_chunks: FxHashSet<ChunkUkey> = Default::default();

    for (entry_name, entrypoint_ukey) in &compilation.entrypoints {
      let Some(entrypoint) = compilation.chunk_group_by_ukey.get(entrypoint_ukey) else {
        continue;
      };

      required_chunks.clear();
      checked_chunk_groups.clear();
      checked_chunks.clear();

      record_chunk_group(
        entry_name,
        &client_entry_modules,
        entrypoint,
        compilation,
        &mut required_chunks,
        &mut checked_chunk_groups,
        &mut checked_chunks,
        plugin_state,
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
    ctx.compiler_hooks.make.tap(make::new(self));

    ctx.compiler_hooks.failed.tap(failed::new(self));

    ctx
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));

    Ok(())
  }
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

  let context = compilation.options.context.clone();
  let mut include_dependencies = vec![];
  for (entry_name, import) in &plugin_state.injected_client_entries {
    {
      if compilation.entries.get(entry_name).is_none() {
        let loader_query = import
          .split_once('?')
          .map(|x| x.1)
          .unwrap_or_default()
          .rsplit_once('!')
          .map(|x| x.0)
          .unwrap_or_default();
        let ParsedClientEntries { modules, .. } = parse_client_entries(loader_query)?;
        compilation.push_diagnostic(Diagnostic::error(
          "RSC Client Entry Mismatch".to_string(),
          format!(
            "Entry '{}' not found in the client compiler. Failed to inject the following client modules: {}",
            entry_name,
            modules
              .into_iter()
              .map(|m| m.request)
              .collect::<Vec<_>>()
              .join(", ")
          ),
        ));
        continue;
      }

      let dependency = Box::new(EntryDependency::new(
        import.clone(),
        context.clone(),
        None,
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

    #[allow(clippy::unwrap_used)]
    let entry_data = compilation.entries.get_mut(entry_name).unwrap();
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

  let Some(mut plugin_state) = PLUGIN_STATES.get_mut(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let start = logger.time("create client reference manifest");
  self
    .traverse_modules(compilation, &mut plugin_state)
    .await?;
  logger.time_end(start);

  let start = logger.time("record entry js files");
  collect_entry_js_files(compilation, &mut plugin_state).await?;
  logger.time_end(start);

  for (entry_name, client_entries) in self.client_entries_per_entry.borrow().iter() {
    let client_actions = collect_client_actions_from_dependencies(compilation, client_entries);
    plugin_state
      .client_actions_per_entry
      .insert(entry_name.clone(), client_actions);
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
