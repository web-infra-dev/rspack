use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilationAfterProcessAssets,
  CompilationParams, CompilerCompilation, CompilerFailed, CompilerMake, CrossOriginLoading,
  DependenciesBlock, Dependency, DependencyId, DependencyType, Logger, ModuleGraph, ModuleId,
  ModuleIdentifier, ModuleType, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_mf::ConsumeSharedModule;
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  Coordinator,
  plugin_state::{ActionIdNamePair, PLUGIN_STATES, PluginState},
  reference_manifest::{
    ClientReferenceManifestEntry, ClientReferenceResolution, CrossOriginMode, ManifestExport,
    ModuleLoading,
  },
  rsc_entry_dependency::RscEntryDependency,
  rsc_entry_module::RscEntryModule,
  rsc_entry_module_factory::RscEntryModuleFactory,
  utils::{
    encode_uri_path, extract_shared_package_from_consume_request, get_canonical_module_resource,
    is_css_mod, is_federation_virtual_module,
  },
};

pub struct RscClientPluginOptions {
  pub coordinator: Arc<Coordinator>,
}

#[plugin]
#[derive(Debug)]
pub struct RscClientPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
  client_entries_per_entry: AtomicRefCell<FxHashMap<String, FxHashSet<DependencyId>>>,
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

fn merge_manifest_export(existing: &mut ManifestExport, mut incoming: ManifestExport) {
  existing.chunks.append(&mut incoming.chunks);
  existing.chunks.sort();
  existing.chunks.dedup();
  existing.r#async = match (existing.r#async, incoming.r#async) {
    (Some(true), _) | (_, Some(true)) => Some(true),
    (Some(false), Some(false)) => Some(false),
    (Some(false), None) | (None, Some(false)) => Some(false),
    (None, None) => None,
  };
}

fn insert_or_merge_manifest_export(
  key: &str,
  mut manifest_export: ManifestExport,
  module_priority: u8,
  plugin_state: &mut PluginState,
) {
  manifest_export.chunks.sort();
  manifest_export.chunks.dedup();
  let key_string = key.to_string();
  let existing_priority = plugin_state
    .client_module_priorities
    .get(&key_string)
    .copied()
    .unwrap_or(u8::MAX);

  if existing_priority == u8::MAX {
    plugin_state
      .client_modules
      .insert(key_string.clone(), manifest_export);
    plugin_state
      .client_module_priorities
      .insert(key_string, module_priority);
    return;
  }

  if module_priority < existing_priority {
    plugin_state
      .client_modules
      .insert(key_string.clone(), manifest_export);
    plugin_state
      .client_module_priorities
      .insert(key_string, module_priority);
    return;
  }

  let Some(existing) = plugin_state.client_modules.get_mut(&key_string) else {
    plugin_state
      .client_modules
      .insert(key_string.clone(), manifest_export);
    plugin_state
      .client_module_priorities
      .insert(key_string, module_priority);
    return;
  };

  if module_priority == existing_priority {
    if existing.id == manifest_export.id {
      merge_manifest_export(existing, manifest_export);
      return;
    }

    if existing.chunks.is_empty() && !manifest_export.chunks.is_empty() {
      *existing = manifest_export;
    }
    return;
  }

  if existing.id == manifest_export.id {
    merge_manifest_export(existing, manifest_export);
  }
}

fn insert_client_reference(
  key: &str,
  client_reference: ClientReferenceManifestEntry,
  module_priority: u8,
  plugin_state: &mut PluginState,
) {
  let key_string = key.to_string();
  let existing_priority = plugin_state
    .client_reference_priorities
    .get(&key_string)
    .copied()
    .unwrap_or(u8::MAX);

  if existing_priority == u8::MAX || module_priority < existing_priority {
    plugin_state
      .client_references
      .insert(key_string.clone(), client_reference);
    plugin_state
      .client_reference_priorities
      .insert(key_string, module_priority);
    return;
  }

  if module_priority == existing_priority {
    let should_replace = plugin_state
      .client_references
      .get(&key_string)
      .is_none_or(|existing| existing != &client_reference);
    if should_replace {
      plugin_state
        .client_references
        .insert(key_string.clone(), client_reference);
      plugin_state
        .client_reference_priorities
        .insert(key_string, module_priority);
    }
  }
}

fn get_shared_resolution(module: &dyn rspack_core::Module) -> Option<ClientReferenceResolution> {
  let consume_shared_module = module.as_any().downcast_ref::<ConsumeSharedModule>()?;
  Some(ClientReferenceResolution::Shared {
    share_key: consume_shared_module.share_key().to_string(),
    share_scope: if consume_shared_module.share_scope().is_empty() {
      vec!["default".to_string()]
    } else {
      consume_shared_module.share_scope().to_vec()
    },
  })
}

fn get_client_reference_resolution(
  module: &dyn rspack_core::Module,
  alias_request: Option<&str>,
  is_shared_alias: bool,
) -> ClientReferenceResolution {
  if is_shared_alias {
    return ClientReferenceResolution::Shared {
      share_key: alias_request.unwrap_or_default().to_string(),
      share_scope: vec!["default".to_string()],
    };
  }

  get_shared_resolution(module).unwrap_or(ClientReferenceResolution::Local)
}

fn normalize_request_for_resource_matching(request: &str) -> Option<String> {
  let request_without_query = request.split('?').next().unwrap_or(request);
  if request_without_query.is_empty() {
    return None;
  }
  extract_shared_package_from_consume_request(request_without_query).or_else(|| {
    if request_without_query.is_empty() {
      None
    } else {
      Some(request_without_query.to_string())
    }
  })
}

fn is_filtered_shared_request(module_request: &str) -> bool {
  matches!(module_request, "react" | "react-dom" | "react-dom/server")
}

fn request_matches_resource(request: &str, resource: &str) -> bool {
  if request == resource {
    return true;
  }

  let Some(normalized_request) = normalize_request_for_resource_matching(request) else {
    return false;
  };
  if normalized_request == resource {
    return true;
  }
  if normalized_request.starts_with('.') || normalized_request.starts_with('/') {
    return false;
  }

  resource.contains(&format!("/{normalized_request}/"))
}

fn module_matches_injected_request(
  entry_name: &str,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
  plugin_state: &PluginState,
) -> bool {
  let Some(injected_modules) = plugin_state.injected_client_entries.get(entry_name) else {
    return false;
  };

  let resource = get_canonical_module_resource(compilation, module);
  if resource.is_empty() {
    return false;
  }

  injected_modules.iter().any(|client_module| {
    let Some(request) = normalize_request_for_resource_matching(client_module.request.as_str())
    else {
      return false;
    };
    if is_filtered_shared_request(&request) {
      return false;
    }
    request_matches_resource(&request, &resource)
  })
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

  let resource = get_canonical_module_resource(compilation, module.as_ref());
  if resource.is_empty() && !is_federation_virtual_module(module.as_ref()) {
    return;
  }

  if is_css_mod(module.as_ref()) {
    let (Some(chunk), Some(entry_css_imports)) = (
      compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get(chunk_ukey),
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
      if imports.get(&resource).is_some() {
        entry_css_files
          .entry(server_entry.clone())
          .or_default()
          .extend(css_files.iter().cloned());
      }
    }
    return;
  }

  let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, module_identifier);
  let mut has_wildcard_client_ref = false;
  let mut matched_request_aliases: FxHashMap<String, bool> = Default::default();
  let mut client_ref_exports = module
    .build_info()
    .rsc
    .as_ref()
    .map(|rsc_meta| {
      has_wildcard_client_ref = rsc_meta.client_refs.iter().any(|name| *name == "*");
      let mut refs = rsc_meta
        .client_refs
        .iter()
        .filter(|name| *name != "*" && *name != "__esModule")
        .map(|name| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
      refs.sort();
      refs.dedup();
      refs
    })
    .unwrap_or_default();

  if client_ref_exports.is_empty()
    && let Some(injected_modules) = plugin_state.injected_client_entries.get(entry_name)
  {
    for client_module in injected_modules {
      if client_module.ids.is_empty() {
        continue;
      }
      let Some(request) = normalize_request_for_resource_matching(client_module.request.as_str())
      else {
        continue;
      };
      if is_filtered_shared_request(&request) {
        continue;
      }
      if !request_matches_resource(&request, &resource) {
        continue;
      }
      matched_request_aliases
        .entry(request)
        .and_modify(|is_shared_alias| *is_shared_alias |= client_module.is_remote)
        .or_insert(client_module.is_remote);
      for id in &client_module.ids {
        if id == "*" {
          has_wildcard_client_ref = true;
          continue;
        }
        if id == "__esModule" {
          continue;
        }
        client_ref_exports.push(id.clone());
      }
    }
    client_ref_exports.sort();
    client_ref_exports.dedup();
  }
  if has_wildcard_client_ref && !client_ref_exports.iter().any(|name| name == "default") {
    client_ref_exports.push("default".to_string());
  }
  client_ref_exports.sort();
  client_ref_exports.dedup();

  let mut non_default_exports = client_ref_exports
    .iter()
    .filter(|export_name| export_name.as_str() != "default")
    .cloned()
    .collect::<Vec<_>>();
  non_default_exports.sort();
  non_default_exports.dedup();

  let manifest_export_name = if non_default_exports.len() == 1 {
    non_default_exports[0].clone()
  } else if client_ref_exports.len() == 1 {
    client_ref_exports[0].clone()
  } else {
    "*".to_string()
  };

  let mut manifest_chunks = required_chunks.to_vec();
  if manifest_chunks.is_empty() && resource.starts_with("mf://") {
    for chunk_ukey in compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(module.identifier())
      .iter()
    {
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
      for file in chunk
        .files()
        .iter()
        .filter(|f| f.ends_with(".js") || f.ends_with(".mjs") || f.ends_with(".cjs"))
      {
        if let Some(asset) = compilation.assets().get(file) {
          let asset_info = asset.get_info();
          if asset_info.hot_module_replacement.unwrap_or(false)
            || asset_info.development.unwrap_or(false)
          {
            continue;
          }
        }
        manifest_chunks.push(chunk_id.to_string());
        manifest_chunks.push(encode_uri_path(file));
      }
    }
    manifest_chunks.sort();
    manifest_chunks.dedup();
  }

  let base_manifest_export = ManifestExport {
    id: module_id.to_string(),
    name: manifest_export_name,
    chunks: manifest_chunks,
    r#async: Some(is_async),
  };
  let module_priority = match module.module_type() {
    ModuleType::Remote => 0,
    ModuleType::Fallback => 1,
    ModuleType::ConsumeShared => 3,
    ModuleType::ProvideShared | ModuleType::ShareContainerShared | ModuleType::SelfReference => 4,
    _ => 2,
  };

  insert_or_merge_manifest_export(
    &resource,
    base_manifest_export.clone(),
    module_priority,
    plugin_state,
  );
  insert_client_reference(
    &resource,
    ClientReferenceManifestEntry {
      export_name: base_manifest_export.name.clone(),
      module_id: base_manifest_export.id.clone(),
      chunks: base_manifest_export.chunks.clone(),
      r#async: base_manifest_export.r#async,
      resolution: get_client_reference_resolution(module.as_ref(), None, false),
    },
    module_priority,
    plugin_state,
  );

  for export_name in &client_ref_exports {
    let export_manifest_export = ManifestExport {
      id: base_manifest_export.id.clone(),
      name: export_name.clone(),
      chunks: base_manifest_export.chunks.clone(),
      r#async: base_manifest_export.r#async,
    };
    insert_or_merge_manifest_export(
      &format!("{resource}#{export_name}"),
      export_manifest_export,
      module_priority,
      plugin_state,
    );
    insert_client_reference(
      &format!("{resource}#{export_name}"),
      ClientReferenceManifestEntry {
        export_name: export_name.clone(),
        module_id: base_manifest_export.id.clone(),
        chunks: base_manifest_export.chunks.clone(),
        r#async: base_manifest_export.r#async,
        resolution: get_client_reference_resolution(module.as_ref(), None, false),
      },
      module_priority,
      plugin_state,
    );
  }

  for (alias_request, is_shared_alias) in matched_request_aliases {
    if alias_request == resource {
      continue;
    }
    insert_or_merge_manifest_export(
      &alias_request,
      base_manifest_export.clone(),
      module_priority,
      plugin_state,
    );
    insert_client_reference(
      &alias_request,
      ClientReferenceManifestEntry {
        export_name: base_manifest_export.name.clone(),
        module_id: base_manifest_export.id.clone(),
        chunks: base_manifest_export.chunks.clone(),
        r#async: base_manifest_export.r#async,
        resolution: get_client_reference_resolution(
          module.as_ref(),
          Some(alias_request.as_str()),
          is_shared_alias,
        ),
      },
      module_priority,
      plugin_state,
    );
    for export_name in &client_ref_exports {
      let export_manifest_export = ManifestExport {
        id: base_manifest_export.id.clone(),
        name: export_name.clone(),
        chunks: base_manifest_export.chunks.clone(),
        r#async: base_manifest_export.r#async,
      };
      insert_or_merge_manifest_export(
        &format!("{alias_request}#{export_name}"),
        export_manifest_export,
        module_priority,
        plugin_state,
      );
      insert_client_reference(
        &format!("{alias_request}#{export_name}"),
        ClientReferenceManifestEntry {
          export_name: export_name.clone(),
          module_id: base_manifest_export.id.clone(),
          chunks: base_manifest_export.chunks.clone(),
          r#async: base_manifest_export.r#async,
          resolution: get_client_reference_resolution(
            module.as_ref(),
            Some(alias_request.as_str()),
            is_shared_alias,
          ),
        },
        module_priority,
        plugin_state,
      );
    }
  }
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
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(chunk_ukey);
    for module_identifier in chunk_modules {
      let Some(module) = module_graph.module_by_identifier(module_identifier) else {
        continue;
      };
      if !client_entry_modules.contains(module_identifier)
        && !module_matches_injected_request(entry_name, module.as_ref(), compilation, plugin_state)
      {
        continue;
      }
      let Some(module_id) =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
      else {
        continue;
      };

      if let Some(concatenated_module) = module.as_concatenated_module() {
        let root_identifier = concatenated_module.get_root();
        let root_module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, root_identifier)
            .unwrap_or(module_id);
        record_module(
          entry_name,
          root_module_id,
          &root_identifier,
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

fn collect_entry_js_files(compilation: &Compilation, plugin_state: &mut PluginState) -> Result<()> {
  for (entry_name, chunk_group_ukey) in &compilation.build_chunk_graph_artifact.entrypoints {
    let Some(chunk_group) = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(chunk_group_ukey)
    else {
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
      .map(|file| format!("{prefix}{file}"))
      .collect::<FxIndexSet<String>>();
  }
  Ok(())
}

fn collect_actions(
  compilation: &Compilation,
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  collected_actions: &mut FxHashMap<String, Vec<ActionIdNamePair>>,
  visited_modules: &mut FxHashSet<ModuleIdentifier>,
) {
  let module = match module_graph.module_by_identifier(module_identifier) {
    Some(m) => m,
    None => return,
  };

  let module_resource = get_canonical_module_resource(compilation, module.as_ref());
  if module_resource.is_empty() && !is_federation_virtual_module(module.as_ref()) {
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

    collected_actions.insert(module_resource, pairs);
  }

  // Collect used exported actions transversely.
  for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
    let Some(resolved_module) = module_graph.get_resolved_module(dependency_id) else {
      continue;
    };
    collect_actions(
      compilation,
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
        compilation,
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
    Self::new_inner(options.coordinator, Default::default())
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

  let plugin_state = PLUGIN_STATES.get(&server_compiler_id).ok_or_else(|| {
    rspack_error::error!(
      "RscClientPlugin: Plugin state not found in make hook for compiler {:#?}.",
      compilation.compiler_id()
    )
  })?;

  let mut include_dependencies = vec![];
  for (entry_name, client_modules) in &plugin_state.injected_client_entries {
    {
      if compilation.entries.get(entry_name).is_none() {
        compilation.push_diagnostic(Diagnostic::warn(
          "RSC Client Entry Mismatch".to_string(),
          format!(
            "Entry '{}' not found in the client compiler. Skipping injection of client modules: {}",
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

      let dependency = Box::new(RscEntryDependency::new(
        entry_name.clone(),
        client_modules.clone(),
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
  self.traverse_modules(compilation, &mut plugin_state)?;
  logger.time_end(start);

  let start = logger.time("record entry js files");
  collect_entry_js_files(compilation, &mut plugin_state)?;
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
