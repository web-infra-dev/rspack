use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilationProcessAssets,
  CompilerMake, CrossOriginLoading, Dependency, DependencyId, EntryDependency, Logger, ModuleGraph,
  ModuleGraphRef, ModuleId, ModuleIdentifier, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  Coordinator,
  loaders::client_entry_loader::CLIENT_ENTRY_LOADER_IDENTIFIER,
  plugin_state::{ActionIdNamePair, PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  reference_manifest::{CrossOriginMode, ManifestExport, ModuleLoading},
  utils::{get_module_resource, is_css_mod},
};

pub struct RscClientPluginOptions {
  pub coordinator: Coordinator,
}

#[plugin]
#[derive(Debug)]
pub struct RscClientPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
  client_entries_per_entry: AtomicRefCell<FxHashMap<String, FxHashSet<DependencyId>>>,
}

fn get_required_chunks(chunk_group: &ChunkGroup, compilation: &Compilation) -> Vec<String> {
  let mut required_chunks = vec![];
  for chunk_ukey in &chunk_group.chunks {
    let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) else {
      continue;
    };
    let Some(chunk_id) = chunk.id(&compilation.chunk_ids_artifact) else {
      continue;
    };
    for file in chunk.files() {
      required_chunks.push(chunk_id.to_string());
      // TODO: encode URI path
      required_chunks.push(file.to_string());
    }
  }
  required_chunks
}

fn record_module(
  entry_name: &str,
  module_id: &ModuleId,
  module_identifier: &ModuleIdentifier,
  client_reference_modules: &FxHashSet<ModuleIdentifier>,
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation,
  required_chunks: &Vec<String>,
  plugin_state: &mut PluginState,
) {
  let Some(normal_module) = (client_reference_modules.contains(module_identifier))
    .then(|| compilation.module_by_identifier(module_identifier))
    .flatten()
    .and_then(|m| m.as_normal_module())
  else {
    return;
  };

  if is_css_mod(normal_module) {
    let (Some(chunk), Some(entry_css_imports)) = (
      compilation.chunk_by_ukey.get(chunk_ukey),
      plugin_state.entry_css_imports.get(entry_name),
    ) else {
      return;
    };

    let prefix = &plugin_state.module_loading.as_ref().unwrap().prefix;
    let css_files: Vec<String> = chunk
      .files()
      .iter()
      .filter(|file| file.ends_with(".css"))
      .map(|file| format!("{}{}", prefix, file))
      .collect();
    if css_files.is_empty() {
      return;
    }

    let entry_css_files = plugin_state
      .entry_css_files
      .entry(entry_name.to_string())
      .or_default();

    let resource = get_module_resource(normal_module);
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

  let resource = normal_module
    .resource_resolved_data()
    .resource()
    .to_string();
  if resource.is_empty() {
    return;
  }

  let is_async = ModuleGraph::is_async(compilation, module_identifier);
  plugin_state.client_modules.insert(
    resource,
    ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: required_chunks.clone(),
      r#async: Some(is_async),
    },
  );
}

fn record_chunk_group(
  entry_name: &str,
  client_reference_modules: &FxHashSet<ModuleIdentifier>,
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

    let module_graph = compilation.get_module_graph();
    let chunk_modules = compilation
      .chunk_graph
      .get_chunk_modules_identifier(chunk_ukey);
    for module_identifier in chunk_modules {
      let Some(module_id) =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
      else {
        continue;
      };
      let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
        continue;
      };
      if let Some(concatenated_module) = module.as_concatenated_module() {
        for inner_module in concatenated_module.get_modules() {
          record_module(
            entry_name,
            module_id,
            &inner_module.id,
            client_reference_modules,
            chunk_ukey,
            compilation,
            required_chunks,
            plugin_state,
          );
        }
      } else {
        record_module(
          entry_name,
          module_id,
          &module_identifier,
          client_reference_modules,
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
    let child_required_chunks = get_required_chunks(child, compilation);
    let start_len = required_chunks.len();
    required_chunks.extend(child_required_chunks);
    record_chunk_group(
      entry_name,
      client_reference_modules,
      child,
      compilation,
      required_chunks,
      checked_chunk_groups,
      checked_chunks,
      plugin_state,
    );
    required_chunks.drain(start_len..);
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
      .entry(entry_name.to_string())
      .or_default();
    for chunk_ukey in &chunk_group.chunks {
      let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) else {
        continue;
      };
      let prefix = &plugin_state.module_loading.as_ref().unwrap().prefix;
      entry_js_files.extend(
        chunk
          .files()
          .iter()
          .filter(|file| file.ends_with(".js"))
          .map(|file| format!("{}{}", prefix, file)),
      );
    }
  }
  Ok(())
}

fn collect_actions_in_dep(
  compilation: &Compilation,
  module_graph: &ModuleGraphRef<'_>,
  module_identifier: &ModuleIdentifier,
  collected_actions: &mut FxHashMap<String, Vec<ActionIdNamePair>>,
  visited_module: &mut FxHashSet<ModuleIdentifier>,
) {
  let module = match module_graph.module_by_identifier(&module_identifier) {
    Some(m) => m,
    None => return,
  };

  let module_resource = get_module_resource(module.as_ref());
  if module_resource.is_empty() {
    return;
  }

  if visited_module.contains(module_identifier) {
    return;
  }
  visited_module.insert(*module_identifier);

  if let Some(action_ids) = module
    .build_info()
    .rsc
    .as_ref()
    .and_then(|rsc| rsc.action_ids.as_ref())
  {
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
    collect_actions_in_dep(
      compilation,
      module_graph,
      resolved_module,
      collected_actions,
      visited_module,
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
  let mut visited_module: FxHashSet<Identifier> = Default::default();

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
      collect_actions_in_dep(
        compilation,
        &module_graph,
        module_identifier,
        &mut collected_actions,
        &mut visited_module,
      );
    }
  }

  collected_actions
}

impl RscClientPlugin {
  pub fn new(coordinator: Arc<Coordinator>) -> Self {
    Self::new_inner(coordinator, Default::default())
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
        Some(template) => {
          // TODO: 只能是纯字符串，模版也不行
          template.to_string()
        }
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

    let mut client_reference_modules: FxHashSet<ModuleIdentifier> = Default::default();
    let module_graph = compilation.get_module_graph();
    for entry_data in compilation.entries.values() {
      for include_dependencies in &entry_data.include_dependencies {
        let Some(module_identifier) =
          module_graph.module_identifier_by_dependency_id(include_dependencies)
        else {
          continue;
        };
        let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
          continue;
        };
        let Some(normal_module) = module.as_normal_module() else {
          continue;
        };
        if !normal_module
          .user_request()
          .starts_with(CLIENT_ENTRY_LOADER_IDENTIFIER)
        {
          continue;
        }
        for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
          let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
            continue;
          };
          client_reference_modules.insert(*connection.module_identifier());
        }
      }
    }

    for (entry_name, entrypoint_ukey) in &compilation.entrypoints {
      let Some(entrypoint) = compilation.chunk_group_by_ukey.get(entrypoint_ukey) else {
        continue;
      };
      let mut required_chunks = vec![];

      let mut checked_chunk_groups: FxHashSet<ChunkGroupUkey> = Default::default();
      let mut checked_chunks: FxHashSet<ChunkUkey> = Default::default();
      record_chunk_group(
        entry_name,
        &client_reference_modules,
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

    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}

// Execution must occur after EntryPlugin to ensure base entries are established
// before injecting client component entries. Stage 100 ensures proper ordering.
#[plugin_hook(CompilerMake for RscClientPlugin, stage = 100)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  self.coordinator.start_client_entries_compilation().await?;

  let server_compiler_id = self.coordinator.get_server_compiler_id().await?;

  let guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let context = compilation.options.context.clone();
  let mut include_dependencies = vec![];
  for (entry_name, import) in &plugin_state.injected_client_entries {
    {
      if compilation.entries.get(entry_name).is_none() {
        return Err(rspack_error::error!(
          "Missing required entry '{}' in client compiler. \
       RscClientPlugin requires an entry with the same name as the server compiler \
       for rendering the React application in the browser. \
       Client components will be injected into this entry.",
          entry_name,
        ));
      }

      let dependency = Box::new(EntryDependency::new(
        import.to_string(),
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

    let entry_data = compilation.entries.get_mut(entry_name).unwrap();
    entry_data
      .include_dependencies
      .extend(include_dependencies.drain(..));
  }

  Ok(())
}

#[plugin_hook(CompilationProcessAssets for RscClientPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.RscClientPlugin");

  let server_compiler_id = self.coordinator.get_server_compiler_id().await?;

  let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get_mut(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let start = logger.time("create client reference manifest");
  self.traverse_modules(compilation, plugin_state).await?;
  logger.time_end(start);

  let start = logger.time("record entry js files");
  collect_entry_js_files(compilation, plugin_state).await?;
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
