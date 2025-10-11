#![allow(non_snake_case)]

mod asset;
mod data;
mod options;
mod utils;

use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  env,
};

use asset::{
  collect_assets_for_module, collect_assets_from_chunk, collect_usage_files_for_module,
  empty_assets_group, merge_assets_group, normalize_assets_group, promote_primary_assets_to_sync,
  remove_assets,
};
use data::{
  BasicStatsMetaData, ManifestExpose, ManifestRemote, ManifestRoot, ManifestShared,
  RemoteEntryMeta, StatsAssetsGroup, StatsExpose, StatsRemote, StatsRoot, StatsShared, TypesMeta,
};
pub use options::ModuleFederationManifestPluginOptions;
use rspack_core::{
  Compilation, CompilationAsset, CompilationProcessAssets, ExtendedStatsOptions, ModuleIdentifier,
  Plugin, PublicPath, Stats as CoreStats,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use utils::{
  collect_expose_requirements, compose_id_with_separator, ensure_shared_entry, is_hot_file,
  parse_consume_shared_identifier, parse_container_exposes_from_identifier,
  parse_provide_shared_identifier, record_shared_usage, strip_ext,
};

use crate::container::remote_module::RemoteModule;

/// Default file names aligned with module-federation/core
const DEFAULT_MANIFEST_FILE: &str = "mf-manifest.json";
const DEFAULT_STATS_FILE: &str = "mf-stats.json";
#[plugin]
#[derive(Debug)]
pub struct ModuleFederationManifestPlugin {
  options: ModuleFederationManifestPluginOptions,
}
impl ModuleFederationManifestPlugin {
  pub fn new(options: ModuleFederationManifestPluginOptions) -> Self {
    Self::new_inner(options)
  }
  fn manifest_filename(&self) -> String {
    let file_name = self
      .options
      .file_name
      .clone()
      .unwrap_or_else(|| DEFAULT_MANIFEST_FILE.to_string());
    match self.options.file_path.as_ref() {
      Some(p) if !p.is_empty() => format!("{}/{}", p.trim_end_matches('/'), file_name),
      _ => file_name,
    }
  }
  fn stats_filename(&self) -> String {
    let file_name = self
      .options
      .stats_file_name
      .clone()
      .unwrap_or_else(|| DEFAULT_STATS_FILE.to_string());
    match self.options.file_path.as_ref() {
      Some(p) if !p.is_empty() => format!("{}/{}", p.trim_end_matches('/'), file_name),
      _ => file_name,
    }
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
#[plugin_hook(CompilationProcessAssets for ModuleFederationManifestPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let disable_emit_env = env::var("MF_DISABLE_EMIT_STATS").ok().is_some();
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
  let (public_path, get_public_path) = if let Some(getter) = self.options.get_public_path.clone() {
    (None, Some(getter))
  } else {
    let path = match &compilation.options.output.public_path {
      PublicPath::Auto => Some("auto".to_string()),
      PublicPath::Filename(f) => Some(PublicPath::render_filename(compilation, f).await),
    };
    (path, None)
  };
  let types_meta = match &self.options.types_file_name {
    Some(name) if !name.is_empty() => TypesMeta {
      name: name.clone(),
      ..Default::default()
    },
    _ => TypesMeta::default(),
  };
  let meta = BasicStatsMetaData {
    name: container_name.clone(),
    globalName: global_name,
    publicPath: public_path,
    getPublicPath: get_public_path,
    types: types_meta,
    remoteEntry: RemoteEntryMeta {
      name: entry_name,
      path: String::new(),
      r#type: compilation
        .options
        .output
        .library
        .as_ref()
        .map(|l| l.library_type.clone())
        .unwrap_or_else(|| "global".to_string()),
    },
    prefetchInterface: None,
    r#type: None,
  };
  // Collect stats from module graph via Stats API
  let stats = CoreStats::new(&*compilation);
  let options = ExtendedStatsOptions {
    modules: true,
    reasons: true,
    ids: true,
    assets: false,
    chunks: false,
    entrypoints: rspack_core::EntrypointsStatsOption::Bool(true),
    cached_modules: true,
    hash: true,
    chunk_relations: false,
    chunk_groups: false,
    chunk_group_auxiliary: false,
    chunk_group_children: false,
    chunk_modules: false,
    depth: false,
    module_assets: false,
    nested_modules: false,
    optimization_bailout: false,
    provided_exports: false,
    source: false,
    used_exports: false,
    warnings: false,
    errors: false,
  };
  // Gather modules
  let exposes_map = RefCell::new(HashMap::<String, StatsExpose>::new());
  let shared_map = RefCell::new(HashMap::<String, StatsShared>::new());
  let shared_usage_links = RefCell::new(Vec::<(String, String)>::new());
  let provide_module_ids = RefCell::new(HashMap::<String, Vec<ModuleIdentifier>>::new());
  let consume_module_ids = RefCell::new(HashMap::<String, Vec<ModuleIdentifier>>::new());
  let remote_module_ids = RefCell::new(Vec::<ModuleIdentifier>::new());
  let container_entry_module = RefCell::new(None::<ModuleIdentifier>);
  let module_ids_by_name = RefCell::new(HashMap::<String, ModuleIdentifier>::new());
  let self_issued_module_ids = RefCell::new(HashMap::<String, ModuleIdentifier>::new());
  stats.get_modules(&options, |modules| {
    for m in modules {
      let module_identifier = match m.identifier {
        Some(id) => id.clone(),
        None => continue,
      };
      let identifier = module_identifier.to_string();
      let module_type = m.module_type.as_str();
      if let Some(name) = m.name.as_ref() {
        module_ids_by_name
          .borrow_mut()
          .insert(name.to_string(), module_identifier.clone());
        if let Some(issuer_name) = m.issuer_name.as_ref()
          && issuer_name.as_ref() == name.as_ref()
        {
          self_issued_module_ids
            .borrow_mut()
            .insert(name.to_string(), module_identifier.clone());
        }
      }
      if identifier.starts_with("container entry") {
        *container_entry_module.borrow_mut() = Some(module_identifier.clone());
        if let Some(exposes) = parse_container_exposes_from_identifier(&identifier) {
          let mut exposes_map_ref = exposes_map.borrow_mut();
          for (expose_key, import_name, import_file) in exposes {
            let name =
              import_name.unwrap_or_else(|| expose_key.trim_start_matches("./").to_string());
            let id_comp = compose_id_with_separator(&container_name, &name);
            let expose_file_key = strip_ext(&import_file);
            exposes_map_ref
              .entry(expose_file_key.clone())
              .or_insert(StatsExpose {
                path: expose_key.clone(),
                id: id_comp,
                name: name.clone(),
                requires: Vec::new(),
                assets: StatsAssetsGroup::default(),
              });
          }
        }
      }
      if identifier.starts_with("remote ") {
        remote_module_ids
          .borrow_mut()
          .push(module_identifier.clone());
      }
      if module_type == "provide-module" {
        if let Some((pkg, ver)) = parse_provide_shared_identifier(&identifier) {
          provide_module_ids
            .borrow_mut()
            .entry(pkg.clone())
            .or_default()
            .push(module_identifier.clone());
          let mut shared_map_ref = shared_map.borrow_mut();
          let entry = ensure_shared_entry(&mut *shared_map_ref, &container_name, &pkg);
          if entry.version.is_empty() {
            entry.version = ver;
          }
          drop(shared_map_ref);
          record_shared_usage(&shared_usage_links, &pkg, &m);
        }
      } else if module_type == "consume-shared-module" {
        if let Some((pkg, required)) = parse_consume_shared_identifier(&identifier) {
          let mut target_ids: Vec<ModuleIdentifier> = Vec::new();
          if let Some(issuer_name) = m.issuer_name.as_ref() {
            if let Some(target) = self_issued_module_ids.borrow().get(issuer_name.as_ref()) {
              target_ids.push(target.clone());
            } else if let Some(target) = module_ids_by_name.borrow().get(issuer_name.as_ref()) {
              target_ids.push(target.clone());
            }
          }
          if target_ids.is_empty() {
            target_ids.push(module_identifier.clone());
          }
          consume_module_ids
            .borrow_mut()
            .entry(pkg.clone())
            .or_default()
            .extend(target_ids);
          let mut shared_map_ref = shared_map.borrow_mut();
          let entry = ensure_shared_entry(&mut *shared_map_ref, &container_name, &pkg);
          if entry.requiredVersion.is_none() && required.is_some() {
            entry.requiredVersion = required;
          }
          drop(shared_map_ref);
          record_shared_usage(&shared_usage_links, &pkg, &m);
        }
      }
    }
  })?;
  let mut exposes_map = exposes_map.into_inner();
  let mut shared_map = shared_map.into_inner();
  let consume_module_ids = consume_module_ids.into_inner();
  let remote_module_ids = remote_module_ids.into_inner();
  let container_entry_module = container_entry_module.into_inner();
  let shared_usage_links = shared_usage_links.into_inner();
  collect_expose_requirements(&mut shared_map, &mut exposes_map, shared_usage_links);
  if !self.options.disable_assets_analyze {
    let mut aggregated_shared_assets: HashMap<String, StatsAssetsGroup> = HashMap::new();
    for (pkg, module_ids) in &consume_module_ids {
      let entry = aggregated_shared_assets
        .entry(pkg.clone())
        .or_insert_with(empty_assets_group);
      for module_id in module_ids {
        if let Some(assets) = collect_assets_for_module(compilation, module_id, &entry_point_names)
        {
          merge_assets_group(entry, assets);
        }
      }
    }

    let mut shared_asset_files: HashSet<String> = HashSet::default();
    for (pkg, mut assets) in aggregated_shared_assets {
      normalize_assets_group(&mut assets);
      promote_primary_assets_to_sync(&mut assets);
      assets.js.r#async.clear();
      assets.css.r#async.clear();
      shared_asset_files.extend(assets.js.sync.iter().cloned());
      shared_asset_files.extend(assets.css.sync.iter().cloned());
      if let Some(shared_entry) = shared_map.get_mut(&pkg) {
        shared_entry.assets = assets;
      }
    }

    for (expose_file_key, expose) in exposes_map.iter_mut() {
      if let Some(chunk_key) = compilation.named_chunks.get(expose_file_key) {
        let mut assets = collect_assets_from_chunk(compilation, chunk_key, &entry_point_names);
        remove_assets(&mut assets, &shared_asset_files);
        promote_primary_assets_to_sync(&mut assets);
        expose.assets = assets;
      }
    }

    if let Some(module_id) = container_entry_module {
      if let Some(mut entry_assets) =
        collect_assets_for_module(compilation, &module_id, &entry_point_names)
      {
        remove_assets(&mut entry_assets, &shared_asset_files);
        promote_primary_assets_to_sync(&mut entry_assets);
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
    }
  }
  let module_graph = compilation.get_module_graph();
  let mut remote_list = Vec::new();
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
    let used_in =
      collect_usage_files_for_module(compilation, &module_graph, &module_id, &entry_point_names);
    remote_list.push(StatsRemote {
      alias: alias.clone(),
      consumingFederationContainerName: container_name.clone(),
      federationContainerName: alias,
      moduleName: module_name,
      entry: None,
      version: None,
      usedIn: used_in,
    });
  }
  // finalize lists
  let exposes: Vec<StatsExpose> = exposes_map.values().cloned().collect();
  let shared: Vec<StatsShared> = shared_map
    .into_iter()
    .map(|(_k, mut v)| {
      v.usedIn.sort();
      v.usedIn.dedup();
      v
    })
    .collect();
  let stats_root = StatsRoot {
    id: container_name.clone(),
    name: container_name.clone(),
    metaData: meta.clone(),
    shared,
    remotes: remote_list.clone(),
    exposes: exposes.clone(),
  };
  // emit stats
  if !disable_emit_env {
    let stats_json = serde_json::to_string_pretty(&stats_root).expect("serialize stats");
    compilation.emit_asset(
      self.stats_filename(),
      CompilationAsset::new(
        Some(RawStringSource::from(stats_json).boxed()),
        Default::default(),
      ),
    );
  }
  // Build manifest from stats
  let manifest = ManifestRoot {
    id: stats_root.id.clone(),
    name: stats_root.name.clone(),
    metaData: stats_root.metaData.clone(),
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
        hash: s.hash,
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
  if !disable_emit_env {
    let manifest_json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    compilation.emit_asset(
      self.manifest_filename(),
      CompilationAsset::new(
        Some(RawStringSource::from(manifest_json).boxed()),
        Default::default(),
      ),
    );
  }
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
