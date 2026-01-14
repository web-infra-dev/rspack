#![allow(non_snake_case)]

mod asset;
mod data;
mod options;
mod utils;

use std::path::Path;

use asset::{
  collect_assets_for_module, collect_assets_from_chunk, collect_usage_files_for_module,
  empty_assets_group, module_source_path, normalize_assets_group,
};
use data::{
  BasicStatsMetaData, ManifestExpose, ManifestRemote, ManifestRoot, ManifestShared,
  RemoteEntryMeta, StatsAssetsGroup, StatsExpose, StatsRemote, StatsShared,
};
pub use data::{StatsBuildInfo, StatsRoot};
pub use options::{
  ManifestExposeOption, ManifestSharedOption, ModuleFederationManifestPluginOptions,
  RemoteAliasTarget,
};
use rspack_core::{
  Compilation, CompilationAsset, CompilationProcessAssets, ModuleIdentifier, ModuleType, Plugin,
  PublicPath,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use utils::{
  collect_expose_requirements, compose_id_with_separator, ensure_shared_entry, is_hot_file,
  parse_consume_shared_identifier, parse_provide_shared_identifier, record_shared_usage, strip_ext,
};

use crate::container::{container_entry_module::ContainerEntryModule, remote_module::RemoteModule};

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationManifestPlugin {
  options: ModuleFederationManifestPluginOptions,
}
impl ModuleFederationManifestPlugin {
  pub fn new(options: ModuleFederationManifestPluginOptions) -> Self {
    Self::new_inner(options)
  }
}
fn get_remote_entry_name(compilation: &Compilation, container_name: &str) -> Option<String> {
  let chunk_group_ukey = compilation.entrypoints.get(container_name)?;
  let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group_ukey);

  let pick_chunk_file = |chunk_ukey: &rspack_core::ChunkUkey| -> Option<String> {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    chunk
      .files()
      .iter()
      .find(|file| !file.ends_with(".css") && !is_hot_file(file))
      .cloned()
  };

  // Prefer the actual entry chunk if it exists.
  let entry_chunk_file = {
    let entry_chunk_key = chunk_group.get_entrypoint_chunk();
    pick_chunk_file(&entry_chunk_key)
  };
  if entry_chunk_file.is_some() {
    return entry_chunk_file;
  }

  // Fallback to the runtime chunk (some configurations emit the entry file there).
  let runtime_chunk_file = {
    let runtime_chunk_key = chunk_group.get_runtime_chunk(&compilation.chunk_group_by_ukey);
    pick_chunk_file(&runtime_chunk_key)
  };
  if runtime_chunk_file.is_some() {
    return runtime_chunk_file;
  }

  // Finally, search every chunk in the group.
  for chunk_key in chunk_group.chunks.iter() {
    if let Some(file) = pick_chunk_file(chunk_key) {
      return Some(file);
    }
  }
  None
}
#[plugin_hook(CompilationProcessAssets for ModuleFederationManifestPlugin, stage = 0)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Prepare entrypoint names
  let entry_point_names: HashSet<String> = compilation
    .entrypoints
    .keys()
    .map(|k| k.to_string())
    .collect();
  // Build metaData
  let container_name = self
    .options
    .name
    .clone()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| compilation.options.output.unique_name.clone());
  let global_name = self
    .options
    .global_name
    .clone()
    .filter(|s| !s.is_empty())
    .or_else(|| {
      compilation
        .options
        .output
        .library
        .as_ref()
        .and_then(|l| match &l.name {
          Some(rspack_core::LibraryName::NonUmdObject(
            rspack_core::LibraryNonUmdObject::String(s),
          )) => Some(s.clone()),
          _ => None,
        })
    })
    .unwrap_or_else(|| container_name.clone());
  let entry_name = get_remote_entry_name(compilation, &container_name).unwrap_or_default();
  let public_path = match &compilation.options.output.public_path {
    PublicPath::Auto => Some("auto".to_string()),
    PublicPath::Filename(f) => Some(PublicPath::render_filename(compilation, f).await),
  };
  let meta = BasicStatsMetaData {
    name: container_name.clone(),
    globalName: global_name,
    build_info: self.options.build_info.clone(),
    publicPath: public_path,
    remoteEntry: RemoteEntryMeta {
      name: entry_name.clone(),
      path: String::new(),
      r#type: compilation
        .options
        .output
        .library
        .as_ref()
        .map(|l| l.library_type.clone())
        .unwrap_or_else(|| "global".to_string()),
    },
    r#type: None,
  };
  let (exposes, shared, remote_list) = if self.options.disable_assets_analyze {
    let exposes = self
      .options
      .exposes
      .iter()
      .map(|expose| {
        let expose_name = expose.path.trim_start_matches("./").to_string();
        StatsExpose {
          path: expose.path.clone(),
          file: String::new(),
          id: compose_id_with_separator(&container_name, &expose_name),
          name: expose_name,
          requires: Vec::new(),
          assets: StatsAssetsGroup::default(),
        }
      })
      .collect::<Vec<_>>();
    let shared = self
      .options
      .shared
      .iter()
      .map(|shared| StatsShared {
        id: compose_id_with_separator(&container_name, &shared.name),
        name: shared.name.clone(),
        version: shared.version.clone().unwrap_or_default(),
        requiredVersion: shared.required_version.clone(),
        // default singleton to true when not provided by user
        singleton: shared.singleton.or(Some(true)),
        assets: StatsAssetsGroup::default(),
        usedIn: Vec::new(),
        usedExports: Vec::new(),
      })
      .collect::<Vec<_>>();
    let remote_list = self
      .options
      .remote_alias_map
      .iter()
      .map(|(alias, target)| {
        let remote_container_name = if target.name.is_empty() {
          alias.clone()
        } else {
          target.name.clone()
        };
        StatsRemote {
          alias: alias.clone(),
          consumingFederationContainerName: container_name.clone(),
          federationContainerName: remote_container_name.clone(),
          moduleName: remote_container_name,
          entry: target.entry.clone(),
          usedIn: vec!["UNKNOWN".to_string()],
        }
      })
      .collect::<Vec<_>>();
    (exposes, shared, remote_list)
  } else {
    let module_graph = compilation.get_module_graph();
    let should_collect_module = |module_id: &ModuleIdentifier| -> bool {
      module_graph
        .module_by_identifier(module_id)
        .map(|module| {
          !matches!(
            module.module_type(),
            ModuleType::ProvideShared | ModuleType::ConsumeShared | ModuleType::Runtime
          )
        })
        .unwrap_or(false)
    };

    let mut exposes_map: HashMap<String, StatsExpose> = HashMap::default();
    let mut expose_chunk_names: HashMap<String, String> = HashMap::default();
    let mut shared_map: HashMap<String, StatsShared> = HashMap::default();
    let mut shared_usage_links: Vec<(String, String)> = Vec::new();
    let mut shared_module_targets: HashMap<String, HashSet<ModuleIdentifier>> = HashMap::default();
    let mut module_ids_by_name: HashMap<String, ModuleIdentifier> = HashMap::default();
    let mut remote_module_ids: Vec<ModuleIdentifier> = Vec::new();
    let mut container_entry_module: Option<ModuleIdentifier> = None;
    for (_, module) in module_graph.modules().into_iter() {
      let module_identifier = module.identifier();
      if let Some(path) = module_source_path(module, compilation) {
        let stripped = strip_ext(&path);
        if !stripped.is_empty() {
          module_ids_by_name
            .entry(stripped.clone())
            .or_insert(module_identifier);
          if !stripped.starts_with("./") {
            module_ids_by_name
              .entry(format!("./{}", stripped))
              .or_insert(module_identifier);
          }
          if let Some(file_name) = Path::new(&stripped).file_name().and_then(|f| f.to_str()) {
            module_ids_by_name
              .entry(file_name.to_string())
              .or_insert(module_identifier);
            let file_base = strip_ext(file_name);
            if !file_base.is_empty() {
              module_ids_by_name
                .entry(file_base.to_string())
                .or_insert(module_identifier);
            }
          }
        }
      }

      if let Some(container_entry) = module
        .as_ref()
        .as_any()
        .downcast_ref::<ContainerEntryModule>()
      {
        container_entry_module = Some(module_identifier);
        let blocks = module.get_blocks();
        for (index, (expose_key, options)) in container_entry.exposes().iter().enumerate() {
          let expose_name = expose_key.trim_start_matches("./").to_string();
          let Some(import) = options.import.iter().find(|request| !request.is_empty()) else {
            continue;
          };
          let id_comp = compose_id_with_separator(&container_name, &expose_name);
          let expose_file_key = strip_ext(import);
          exposes_map
            .entry(expose_file_key.clone())
            .or_insert(StatsExpose {
              path: expose_key.clone(),
              file: String::new(),
              id: id_comp,
              name: expose_name,
              requires: Vec::new(),
              assets: StatsAssetsGroup::default(),
            });

          if let Some(block_id) = blocks.get(index)
            && let Some(chunk_group) = compilation
              .chunk_graph
              .get_block_chunk_group(block_id, &compilation.chunk_group_by_ukey)
            && let Some(chunk_key) = chunk_group.chunks.first()
            && let Some(chunk) = compilation.chunk_by_ukey.get(chunk_key)
            && let Some(name) = chunk.name()
          {
            expose_chunk_names.insert(expose_file_key.clone(), name.to_string());
          }

          if !expose_chunk_names.contains_key(&expose_file_key)
            && let Some(n) = &options.name
            && !n.is_empty()
          {
            expose_chunk_names.insert(expose_file_key, n.clone());
          }
        }
        continue;
      }

      let module_type = module.module_type();
      let identifier = module_identifier.to_string();

      if matches!(module_type, ModuleType::Remote) {
        remote_module_ids.push(module_identifier);
      }

      if matches!(module_type, ModuleType::ProvideShared) {
        if let Some((pkg, ver)) = parse_provide_shared_identifier(&identifier) {
          let entry = ensure_shared_entry(&mut shared_map, &container_name, &pkg);
          if entry.version.is_empty() {
            entry.version = ver;
          }
          // overlay user-configured shared options (singleton/requiredVersion/version)
          if let Some(opt) = self.options.shared.iter().find(|s| s.name == pkg) {
            if let Some(singleton) = opt.singleton {
              entry.singleton = Some(singleton);
            }
            if entry.requiredVersion.is_none() {
              entry.requiredVersion = opt.required_version.clone();
            }
            if let Some(cfg_ver) = opt.version.clone().filter(|_| entry.version.is_empty()) {
              entry.version = cfg_ver;
            }
          }
          let targets = shared_module_targets.entry(pkg.clone()).or_default();
          for connection in module_graph.get_outgoing_connections(&module_identifier) {
            let referenced = *connection.module_identifier();
            if should_collect_module(&referenced) {
              targets.insert(referenced);
            }
            let resolved = connection.resolved_module;
            if should_collect_module(&resolved) {
              targets.insert(resolved);
            }
          }
          record_shared_usage(
            &mut shared_usage_links,
            &pkg,
            &module_identifier,
            module_graph,
            compilation,
          );
        }
        continue;
      }

      if matches!(module_type, ModuleType::ConsumeShared)
        && let Some((pkg, required)) = parse_consume_shared_identifier(&identifier)
      {
        let mut target_ids: HashSet<ModuleIdentifier> = HashSet::default();
        for connection in module_graph.get_outgoing_connections(&module_identifier) {
          let module_id = *connection.module_identifier();
          if should_collect_module(&module_id) {
            target_ids.insert(module_id);
          }
          let resolved = connection.resolved_module;
          if should_collect_module(&resolved) {
            target_ids.insert(resolved);
          }
        }
        shared_module_targets
          .entry(pkg.clone())
          .or_default()
          .extend(target_ids.into_iter());
        let entry = ensure_shared_entry(&mut shared_map, &container_name, &pkg);
        if entry.requiredVersion.is_none() && required.is_some() {
          entry.requiredVersion = required;
        }
        // overlay user-configured shared options
        if let Some(opt) = self.options.shared.iter().find(|s| s.name == pkg) {
          if let Some(singleton) = opt.singleton {
            entry.singleton = Some(singleton);
          }
          // prefer parsed requiredVersion but fill from config if still None
          if entry.requiredVersion.is_none() {
            entry.requiredVersion = opt.required_version.clone();
          }
          if let Some(cfg_ver) = opt.version.clone().filter(|_| entry.version.is_empty()) {
            entry.version = cfg_ver;
          }
        }
        record_shared_usage(
          &mut shared_usage_links,
          &pkg,
          &module_identifier,
          module_graph,
          compilation,
        );
      }
    }

    let mut expose_module_paths: HashMap<String, String> = HashMap::default();
    for expose_key in exposes_map.keys() {
      if let Some(module_id) = module_ids_by_name.get(expose_key)
        && let Some(module) = module_graph.module_by_identifier(module_id)
        && let Some(path) = module_source_path(module, compilation)
      {
        expose_module_paths.insert(expose_key.clone(), path);
      }
    }

    let shared_usage_links_for_requirements = shared_usage_links.clone();
    collect_expose_requirements(
      &mut shared_map,
      &mut exposes_map,
      shared_usage_links_for_requirements,
      &expose_module_paths,
    );
    let chunk_graph = &compilation.chunk_graph;
    let mut shared_chunk_map: HashMap<String, HashSet<rspack_core::ChunkUkey>> = HashMap::default();
    for (pkg, module_ids) in &shared_module_targets {
      let entry = shared_chunk_map.entry(pkg.clone()).or_default();
      for module_id in module_ids {
        for chunk_ukey in chunk_graph.get_module_chunks(*module_id).iter() {
          entry.insert(*chunk_ukey);
          let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
          for group_ukey in chunk.groups() {
            let group = compilation.chunk_group_by_ukey.expect_get(group_ukey);
            if let Some(name) = group.name()
              && !entry_point_names.contains(name)
            {
              for extra_chunk in group.chunks.iter() {
                entry.insert(*extra_chunk);
              }
            }
          }
        }
      }
    }

    let mut aggregated_shared_assets: HashMap<String, StatsAssetsGroup> = HashMap::default();
    for (pkg, chunk_ids) in shared_chunk_map {
      let entry = aggregated_shared_assets
        .entry(pkg)
        .or_insert_with(empty_assets_group);
      for chunk_ukey in chunk_ids {
        let chunk_assets = collect_assets_from_chunk(compilation, &chunk_ukey, &entry_point_names);
        entry.js.sync.extend(chunk_assets.js.sync);
        entry.css.sync.extend(chunk_assets.css.sync);
      }
    }

    let mut shared_asset_files: HashSet<String> = HashSet::default();
    for (pkg, mut assets) in aggregated_shared_assets {
      normalize_assets_group(&mut assets);
      assets.js.r#async.clear();
      assets.css.r#async.clear();
      shared_asset_files.extend(assets.js.sync.iter().cloned());
      shared_asset_files.extend(assets.css.sync.iter().cloned());
      if let Some(shared_entry) = shared_map.get_mut(&pkg) {
        shared_entry.assets = assets;
      }
    }

    for (expose_file_key, expose) in exposes_map.iter_mut() {
      let mut assets = None;
      if let Some(chunk_name) = expose_chunk_names.get(expose_file_key)
        && let Some(chunk_key) = compilation.named_chunks.get(chunk_name)
      {
        assets = Some(collect_assets_from_chunk(
          compilation,
          chunk_key,
          &entry_point_names,
        ));
      }
      if assets.is_none()
        && let Some(chunk_key) = compilation.named_chunks.get(expose_file_key)
      {
        assets = Some(collect_assets_from_chunk(
          compilation,
          chunk_key,
          &entry_point_names,
        ));
      }
      if assets.is_none()
        && let Some(module_id) = module_ids_by_name.get(expose_file_key)
      {
        assets = collect_assets_for_module(compilation, module_id, &entry_point_names);
      }
      let mut assets = assets.unwrap_or_else(empty_assets_group);
      if let Some(path) = expose_module_paths.get(expose_file_key) {
        expose.file = path.clone();
      }
      if !entry_name.is_empty() {
        assets.js.sync.retain(|asset| asset != &entry_name);
        assets.js.r#async.retain(|asset| asset != &entry_name);
        assets.css.sync.retain(|asset| asset != &entry_name);
        assets.css.r#async.retain(|asset| asset != &entry_name);
      }
      assets
        .js
        .sync
        .retain(|asset| !shared_asset_files.contains(asset));
      assets
        .js
        .r#async
        .retain(|asset| !shared_asset_files.contains(asset));
      assets
        .css
        .sync
        .retain(|asset| !shared_asset_files.contains(asset));
      assets
        .css
        .r#async
        .retain(|asset| !shared_asset_files.contains(asset));
      normalize_assets_group(&mut assets);
      expose.assets = assets;
    }

    if let Some(module_id) = container_entry_module
      && let Some(mut entry_assets) =
        collect_assets_for_module(compilation, &module_id, &entry_point_names)
    {
      entry_assets
        .js
        .sync
        .retain(|asset| !shared_asset_files.contains(asset));
      entry_assets
        .css
        .sync
        .retain(|asset| !shared_asset_files.contains(asset));
      if !entry_name.is_empty() {
        entry_assets.js.sync.retain(|asset| asset != &entry_name);
        entry_assets.js.r#async.retain(|asset| asset != &entry_name);
        entry_assets.css.sync.retain(|asset| asset != &entry_name);
        entry_assets
          .css
          .r#async
          .retain(|asset| asset != &entry_name);
      }
      normalize_assets_group(&mut entry_assets);
      for expose in exposes_map.values_mut() {
        let is_empty = expose.assets.js.sync.is_empty()
          && expose.assets.js.r#async.is_empty()
          && expose.assets.css.sync.is_empty()
          && expose.assets.css.r#async.is_empty();
        if is_empty {
          expose.assets = entry_assets.clone();
        }
      }
    }

    let module_graph = compilation.get_module_graph();
    let mut remote_list = Vec::new();
    let provided_remote_alias_map = self.options.remote_alias_map.clone();
    for module_id in remote_module_ids {
      let Some(module) = compilation.module_by_identifier(&module_id) else {
        continue;
      };
      let Some(remote_module) = module.as_ref().as_any().downcast_ref::<RemoteModule>() else {
        continue;
      };
      let alias = remote_module.remote_key.clone();
      let module_name = {
        let trimmed = remote_module.internal_request.trim_start_matches("./");
        if trimmed.is_empty() {
          remote_module.internal_request.clone()
        } else {
          trimmed.to_string()
        }
      };
      let (entry, federation_container_name) =
        if let Some(target) = provided_remote_alias_map.get(&alias) {
          let remote_container_name = if target.name.is_empty() {
            alias.clone()
          } else {
            target.name.clone()
          };
          (
            target.entry.clone().filter(|entry| !entry.is_empty()),
            remote_container_name,
          )
        } else {
          (None, alias.clone())
        };
      let used_in =
        collect_usage_files_for_module(compilation, module_graph, &module_id, &entry_point_names)
          // keep only the file path, drop aggregated suffix like " + 1 modules"
          .into_iter()
          .map(|s| {
            if let Some((before, _)) = s.split_once(" + ") {
              before.to_string()
            } else {
              s
            }
          })
          .collect();
      remote_list.push(StatsRemote {
        alias: alias.clone(),
        consumingFederationContainerName: container_name.clone(),
        federationContainerName: federation_container_name,
        moduleName: module_name,
        entry,
        usedIn: used_in,
      });
    }

    let exposes = exposes_map.values().cloned().collect::<Vec<_>>();
    let shared = shared_map
      .into_values()
      .map(|mut v| {
        v.usedIn.sort();
        v.usedIn.dedup();
        v
      })
      .collect::<Vec<_>>();
    (exposes, shared, remote_list)
  };
  // Ensure all configured remotes exist in stats, add missing with defaults
  let mut remote_list = remote_list;
  for (alias, target) in self.options.remote_alias_map.iter() {
    if !remote_list.iter().any(|r| r.alias == *alias) {
      let remote_container_name = if target.name.is_empty() {
        alias.clone()
      } else {
        target.name.clone()
      };
      remote_list.push(StatsRemote {
        alias: alias.clone(),
        consumingFederationContainerName: container_name.clone(),
        federationContainerName: remote_container_name.clone(),
        // default moduleName to "." for missing entries
        moduleName: ".".to_string(),
        entry: target.entry.clone(),
        usedIn: vec!["UNKNOWN".to_string()],
      });
    }
  }

  let stats_root = StatsRoot {
    id: container_name.clone(),
    name: container_name.clone(),
    metaData: meta.clone(),
    shared,
    remotes: remote_list.clone(),
    exposes: exposes.clone(),
  };
  // emit stats
  let stats_json = serde_json::to_string_pretty(&stats_root).expect("serialize stats");
  compilation.emit_asset(
    self.options.stats_file_name.clone(),
    CompilationAsset::new(
      Some(RawStringSource::from(stats_json).boxed()),
      Default::default(),
    ),
  );
  // Build manifest from stats
  let mut manifest_meta = stats_root.metaData.clone();
  if let Some(build_info) = &mut manifest_meta.build_info {
    build_info.target = None;
    build_info.plugins = None;
  }
  let manifest = ManifestRoot {
    id: stats_root.id.clone(),
    name: stats_root.name.clone(),
    metaData: manifest_meta,
    exposes: exposes
      .into_iter()
      .map(|e| ManifestExpose {
        id: e.id,
        name: e.name,
        path: e.path,
        assets: e.assets,
      })
      .collect(),
    shared: stats_root
      .shared
      .into_iter()
      .map(|s| ManifestShared {
        id: s.id,
        name: s.name,
        version: s.version,
        requiredVersion: s.requiredVersion,
        singleton: s.singleton,
        assets: s.assets,
      })
      .collect(),
    remotes: remote_list
      .into_iter()
      .map(|r| ManifestRemote {
        federationContainerName: r.federationContainerName,
        moduleName: r.moduleName,
        alias: r.alias,
        entry: r.entry,
      })
      .collect(),
  };
  let manifest_json: String = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
  compilation.emit_asset(
    self.options.manifest_file_name.clone(),
    CompilationAsset::new(
      Some(RawStringSource::from(manifest_json).boxed()),
      Default::default(),
    ),
  );
  Ok(())
}
impl Plugin for ModuleFederationManifestPlugin {
  fn name(&self) -> &'static str {
    "rspack.ModuleFederationManifestPlugin"
  }
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    // Align with webpack's stage: PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
