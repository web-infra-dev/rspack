use std::sync::{Arc, atomic::AtomicI32};

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{Identifiable, IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxModule, ChunkGraph, Compilation, Context, Dependency, DependencyId, DependencyType,
  ExportInfoData, ExportMode, ExportProvided, ExportsInfoArtifact, ExtendedReferencedExport,
  Module, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdsArtifact, ModuleType,
  OptimizationBailoutItem, SideEffectsStateArtifact, UsageState, UsedByExports,
  UsedByExportsCondition, collect_referenced_export_items,
  rspack_sources::{MapOptions, ObjectPool},
};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_javascript::{
  dependency::{ESMExportImportedSpecifierDependency, ESMImportSpecifierDependency, URLDependency},
  has_impure_deferred_pure_checks,
};
use rspack_plugin_json::create_object_for_exports_info;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use thread_local::ThreadLocal;

use crate::{
  ChunkUkey, ModuleKind, ModuleUkey, RsdoctorConnectionsOnlyImport,
  RsdoctorConnectionsOnlyImportConnection, RsdoctorDependency, RsdoctorExportUsageDependency,
  RsdoctorExportUsageEdge, RsdoctorJsonModuleSizes, RsdoctorModule, RsdoctorModuleId,
  RsdoctorModuleOriginalSource, RsdoctorSideEffectLocation,
};

type ExportUsageExport = Option<Vec<String>>;
type ExportUsageExports = Vec<ExportUsageExport>;
type DependencyExportUsage = Vec<(ExportUsageExport, ExportUsageExport)>;

pub fn collect_json_module_sizes(
  modules: &IdentifierMap<&BoxModule>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> RsdoctorJsonModuleSizes {
  let mut json_sizes: RsdoctorJsonModuleSizes = RsdoctorJsonModuleSizes::default();

  for (module_id, module) in modules.iter() {
    if module.module_type() != &ModuleType::Json {
      continue;
    }

    let Some(json_data) = module.build_info().json_data.as_ref() else {
      continue;
    };

    let exports_info = exports_info_artifact.get_exports_info_data(module_id);

    let final_json = match json_data {
      json::JsonValue::Object(_) | json::JsonValue::Array(_) => {
        let needs_tree_shaking = exports_info.other_exports_info().get_used(None)
          == UsageState::Unused
          || exports_info.exports().values().any(|info| {
            let used = info.get_used(None);
            used == UsageState::Unused || used == UsageState::OnlyPropertiesUsed
          });

        if needs_tree_shaking {
          create_object_for_exports_info(
            json_data.clone(),
            exports_info,
            None,
            exports_info_artifact,
          )
        } else {
          json_data.clone()
        }
      }
      _ => json_data.clone(),
    };

    let json_str = json::stringify(final_json);
    let size = ("module.exports = ".len() + json_str.len()) as i32;
    json_sizes.insert(module_id.to_string(), size);
  }

  json_sizes
}

pub fn collect_modules(
  modules: &IdentifierMap<&BoxModule>,
  module_graph: &ModuleGraph,
  chunk_graph: &ChunkGraph,
  context: &Context,
) -> IdentifierMap<RsdoctorModule> {
  let module_ukey_counter: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));

  modules
    .par_iter()
    .map(|(module_id, module)| {
      let ukey = module_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      let depth = module_graph.get_depth(module_id);
      let path = if let Some(module) = module.as_normal_module() {
        module.resource_resolved_data().resource().to_owned()
      } else if let Some(module) = module.as_concatenated_module() {
        let root = module.get_root();
        if let Some(module) = module_graph
          .module_by_identifier(&root)
          .and_then(|m| m.as_normal_module())
        {
          module.resource_resolved_data().resource().to_owned()
        } else {
          root.to_string()
        }
      } else {
        module.readable_identifier(context).to_string()
      };
      let is_concatenated = module.as_concatenated_module().is_some();
      let chunks = chunk_graph
        .try_get_module_chunks(module_id)
        .map(|chunks| {
          chunks
            .iter()
            .map(|i| i.as_u32() as ChunkUkey)
            .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

      (
        module_id.to_owned(),
        RsdoctorModule {
          ukey,
          identifier: module.identifier(),
          path,
          is_entry: depth.is_some_and(|d| d == 0),
          kind: if is_concatenated {
            ModuleKind::Concatenated
          } else {
            ModuleKind::Normal
          },
          layer: module.get_layer().cloned(),
          dependencies: HashSet::default(),
          imported: HashSet::default(),
          modules: HashSet::default(),
          belong_modules: HashSet::default(),
          chunks,
          issuer_path: None,
          bailout_reason: HashSet::default(),
          side_effects: None,
          side_effects_locations: Vec::new(),
          exports_type: module.build_meta().exports_type,
        },
      )
    })
    .collect::<IdentifierMap<_>>()
}

pub fn collect_concatenated_modules(
  modules: &IdentifierMap<&BoxModule>,
) -> (IdentifierMap<IdentifierSet>, IdentifierMap<IdentifierSet>) {
  let children_map = modules
    .par_iter()
    .filter_map(|(module_id, module)| {
      let concatenated_module = module.as_concatenated_module()?;
      Some((
        module_id.to_owned(),
        concatenated_module
          .get_modules()
          .iter()
          .map(|m| m.id)
          .collect::<IdentifierSet>(),
      ))
    })
    .collect::<IdentifierMap<_>>();

  let parent_map = children_map
    .iter()
    .flat_map(|(parent, children)| {
      children
        .iter()
        .map(|child| (*child, *parent))
        .collect::<HashSet<_>>()
    })
    .fold(
      IdentifierMap::default(),
      |mut acc: IdentifierMap<IdentifierSet>, (child, parent)| {
        acc.entry(child).or_default().insert(parent);
        acc
      },
    );

  (children_map, parent_map)
}

pub fn collect_module_original_sources(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
  module_graph: &ModuleGraph,
  compilation: &Compilation,
) -> Vec<RsdoctorModuleOriginalSource> {
  let ifs = compilation.input_filesystem.clone();

  let tls: ThreadLocal<ObjectPool> = ThreadLocal::new();
  modules
    .par_iter()
    .filter_map(|(module_id, module)| {
      let module = if let Some(module) = module.as_concatenated_module() {
        module_graph
          .module_by_identifier(&module.get_root())?
          .as_normal_module()?
      } else {
        module.as_normal_module()?
      };
      let resource = module.resource_resolved_data().resource().to_owned();
      let module_ukey = module_ukeys.get(module_id)?;
      let object_pool = tls.get_or(ObjectPool::default);
      let source = module
        .source()
        .and_then(|s| s.map(object_pool, &MapOptions::default()))
        .and_then(|s| {
          let idx = s.sources().iter().position(|s| s.eq(&resource))?;
          let source = s.sources_content().get(idx)?;
          Some(RsdoctorModuleOriginalSource {
            module: *module_ukey,
            source: source.to_string(),
            size: source.len() as i32,
          })
        })
        .or_else(|| {
          let resource = Utf8PathBuf::from(resource);
          let buffer = ifs.read_sync(&resource).ok()?;
          let content = String::from_utf8(buffer).ok()?;
          Some(RsdoctorModuleOriginalSource {
            module: *module_ukey,
            size: content.len() as i32,
            source: content,
          })
        })?;

      let mut source = source;

      let (map, result_map) = compilation.code_generation_results.inner();
      let module_identifier = module.identifier();
      let code_gen_key = if map.contains_key(&module_identifier) {
        &module_identifier
      } else {
        module_id
      };

      if let Some(entry) = map.get(code_gen_key)
        && let Some(id) = entry.values().next()
        && let Some(res) = result_map.get(id)
      {
        source.size = res.inner().values().map(|s| s.size() as i32).sum();
      }

      Some(source)
    })
    .collect::<Vec<_>>()
}

pub fn collect_module_dependencies(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
  module_graph: &ModuleGraph,
) -> IdentifierMap<IdentifierMap<(DependencyId, RsdoctorDependency)>> {
  let dependency_ukey_counter = Arc::new(AtomicI32::new(0));

  modules
    .par_iter()
    .filter_map(|(module_id, _)| {
      let rsd_module_ukey = module_ukeys.get(module_id)?;
      let dependencies = module_graph
        .get_outgoing_connections(module_id)
        .filter_map(|conn| {
          let dep = module_graph
            .dependency_by_id(&conn.dependency_id)
            .as_module_dependency()?;

          if matches!(
            dep.dependency_type(),
            DependencyType::CjsSelfReference
              | DependencyType::EsmExportImportedSpecifier
              | DependencyType::EsmImportSpecifier
          ) {
            return None;
          }

          let dep_module = module_graph
            .module_identifier_by_dependency_id(&conn.dependency_id)
            .and_then(|mid| module_ukeys.get(mid).map(|ukey| (*mid, *ukey)))?;

          let dep_ukey = dependency_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
          Some((
            dep_module.0,
            (
              conn.dependency_id,
              RsdoctorDependency {
                ukey: dep_ukey,
                kind: *dep.dependency_type(),
                request: dep.user_request().into(),
                module: *rsd_module_ukey,
                dependency: dep_module.1,
              },
            ),
          ))
        })
        .collect::<IdentifierMap<(DependencyId, RsdoctorDependency)>>();

      Some((module_id.to_owned(), dependencies))
    })
    .collect::<IdentifierMap<IdentifierMap<(DependencyId, RsdoctorDependency)>>>()
}

#[inline(never)]
fn get_origin_exports(used_by_exports: Option<&UsedByExports>) -> Vec<Option<Vec<String>>> {
  match used_by_exports.map(|used_by_exports| used_by_exports.condition()) {
    None | Some(UsedByExportsCondition::Bool(true)) => vec![None],
    Some(UsedByExportsCondition::Bool(false)) => {
      if used_by_exports
        .is_some_and(|used_by_exports| !used_by_exports.deferred_pure_checks().is_empty())
      {
        vec![None]
      } else {
        vec![]
      }
    }
    Some(UsedByExportsCondition::Set(exports)) => exports
      .iter()
      .map(|export| Some(vec![export.to_string()]))
      .collect::<Vec<_>>(),
  }
}

fn get_esm_import_specifier_target_exports(
  dependency: &ESMImportSpecifierDependency,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<Option<Vec<String>>> {
  dependency
    .get_referenced_exports(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      None,
    )
    .into_iter()
    .map(|referenced_export| match referenced_export {
      ExtendedReferencedExport::Array(ids) => ids,
      ExtendedReferencedExport::Export(export) => export.name,
    })
    .map(|ids| {
      if ids.is_empty() {
        None
      } else {
        Some(ids.into_iter().map(|id| id.to_string()).collect())
      }
    })
    .collect()
}

#[inline(never)]
fn cross_export_usage(
  origin_exports: ExportUsageExports,
  target_exports: ExportUsageExports,
) -> DependencyExportUsage {
  let mut export_usage = Vec::with_capacity(origin_exports.len() * target_exports.len());
  for origin_export in origin_exports {
    for target_export in &target_exports {
      export_usage.push((origin_export.clone(), target_export.clone()));
    }
  }
  export_usage
}

#[inline(never)]
fn dependency_export_usage(
  dependency: &dyn Dependency,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<DependencyExportUsage> {
  if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>() {
    return Some(cross_export_usage(
      get_origin_exports(dependency.used_by_exports()),
      get_esm_import_specifier_target_exports(
        dependency,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      ),
    ));
  }
  if let Some(dependency) = dependency.downcast_ref::<ESMExportImportedSpecifierDependency>() {
    return Some(get_esm_export_imported_specifier_exports(
      dependency,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    ));
  }
  if let Some(dependency) = dependency.downcast_ref::<URLDependency>() {
    return Some(cross_export_usage(
      get_origin_exports(dependency.used_by_exports()),
      vec![None],
    ));
  }
  None
}

fn get_esm_export_imported_specifier_target_exports(
  dependency: &ESMExportImportedSpecifierDependency,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<Option<Vec<String>>> {
  let ids = dependency.get_ids(module_graph);
  if dependency.name.is_some() {
    let mode = dependency.get_mode(
      module_graph,
      None,
      module_graph_cache,
      exports_info_artifact,
    );
    match mode {
      ExportMode::Missing
      | ExportMode::LazyMake
      | ExportMode::Unused(_)
      | ExportMode::EmptyStar(_)
      | ExportMode::ReexportUndefined(_) => {
        return vec![];
      }
      ExportMode::ReexportDynamicDefault(_) | ExportMode::DynamicReexport(_) => {
        return vec![None];
      }
      ExportMode::ReexportNamedDefault(mode) => {
        return collect_referenced_target_exports(
          exports_info_artifact,
          mode
            .partial_namespace_export_info
            .as_data(exports_info_artifact),
          false,
        );
      }
      ExportMode::ReexportNamespaceObject(mode) => {
        return collect_referenced_target_exports(
          exports_info_artifact,
          mode
            .partial_namespace_export_info
            .as_data(exports_info_artifact),
          false,
        );
      }
      ExportMode::ReexportFakeNamespaceObject(mode) => {
        return collect_referenced_target_exports(
          exports_info_artifact,
          mode
            .partial_namespace_export_info
            .as_data(exports_info_artifact),
          true,
        );
      }
      ExportMode::NormalReexport(mode) => {
        let mut referenced_exports = Vec::new();
        let target_exports_info = module_graph
          .module_identifier_by_dependency_id(&dependency.id)
          .map(|identifier| exports_info_artifact.get_exports_info_data(identifier));
        for item in &mode.items {
          if item.hidden {
            continue;
          }
          if target_exports_info.is_some_and(|target_exports_info| {
            matches!(
              target_exports_info.is_export_provided(exports_info_artifact, &item.ids),
              Some(ExportProvided::NotProvided)
            )
          }) {
            continue;
          }
          collect_referenced_export_items(
            exports_info_artifact,
            None,
            &mut referenced_exports,
            item.ids.iter().collect(),
            Some(item.export_info.as_data(exports_info_artifact)),
            false,
            &mut Default::default(),
          );
        }
        return map_referenced_target_exports(referenced_exports);
      }
    }
  }
  if ids.is_empty() {
    vec![None]
  } else {
    vec![Some(ids.iter().map(|id| id.to_string()).collect())]
  }
}

fn collect_referenced_target_exports(
  exports_info_artifact: &ExportsInfoArtifact,
  export_info: &ExportInfoData,
  default_points_to_self: bool,
) -> Vec<Option<Vec<String>>> {
  let mut referenced_exports = Vec::new();
  collect_referenced_export_items(
    exports_info_artifact,
    None,
    &mut referenced_exports,
    vec![],
    Some(export_info),
    default_points_to_self,
    &mut Default::default(),
  );
  map_referenced_target_exports(referenced_exports)
}

fn map_referenced_target_exports(referenced_exports: Vec<Vec<&Atom>>) -> Vec<Option<Vec<String>>> {
  referenced_exports
    .into_iter()
    .map(|ids| {
      if ids.is_empty() {
        None
      } else {
        Some(ids.into_iter().map(|id| id.to_string()).collect())
      }
    })
    .collect()
}

fn get_esm_star_reexport_hidden_exports(
  dependency: &ESMExportImportedSpecifierDependency,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<HashSet<Atom>> {
  match dependency.get_mode(
    module_graph,
    None,
    module_graph_cache,
    exports_info_artifact,
  ) {
    ExportMode::NormalReexport(mode) => {
      let hidden = mode
        .items
        .into_iter()
        .filter_map(|item| item.hidden.then_some(item.name))
        .collect::<HashSet<_>>();
      (!hidden.is_empty()).then_some(hidden)
    }
    ExportMode::DynamicReexport(mode) => mode.hidden,
    ExportMode::EmptyStar(mode) => mode.hidden,
    _ => None,
  }
}

#[inline(never)]
fn get_esm_export_imported_specifier_exports(
  dependency: &ESMExportImportedSpecifierDependency,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> DependencyExportUsage {
  if let Some(name) = &dependency.name {
    return cross_export_usage(
      vec![Some(vec![name.to_string()])],
      get_esm_export_imported_specifier_target_exports(
        dependency,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      ),
    );
  }

  let Some(origin_module_identifier) = module_graph.get_parent_module(&dependency.id) else {
    return vec![(None, None)];
  };
  let origin_exports_info = exports_info_artifact.get_exports_info_data(origin_module_identifier);
  if origin_exports_info.other_exports_info().get_used(None) != UsageState::Unused {
    return vec![(None, None)];
  }
  let active_exports = dependency.active_exports(module_graph);
  let target_exports_info = module_graph
    .module_identifier_by_dependency_id(&dependency.id)
    .map(|identifier| exports_info_artifact.get_exports_info_data(identifier));
  let hidden_exports = get_esm_star_reexport_hidden_exports(
    dependency,
    module_graph,
    module_graph_cache,
    exports_info_artifact,
  );
  origin_exports_info
    .exports()
    .values()
    .filter_map(|export_info| {
      let name = export_info.name()?;
      if name == "default"
        || active_exports.contains(name)
        || hidden_exports
          .as_ref()
          .is_some_and(|hidden_exports| hidden_exports.contains(name))
        || export_info.get_used(None) == UsageState::Unused
        || target_exports_info.is_some_and(|target_exports_info| {
          matches!(
            target_exports_info
              .is_export_provided(exports_info_artifact, std::slice::from_ref(name)),
            Some(ExportProvided::NotProvided)
          )
        })
      {
        return None;
      }
      let mut referenced_exports = Vec::new();
      collect_referenced_export_items(
        exports_info_artifact,
        None,
        &mut referenced_exports,
        vec![name],
        Some(export_info),
        false,
        &mut Default::default(),
      );
      Some(map_referenced_target_exports(referenced_exports))
    })
    .flatten()
    .map(|export| (export.clone(), export))
    .collect()
}

pub fn collect_export_usage_dependencies(
  modules: &IdentifierMap<&BoxModule>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<RsdoctorExportUsageDependency> {
  modules
    .keys()
    .flat_map(|module_id| {
      module_graph
        .get_outgoing_connections(module_id)
        .flat_map(|conn| {
          let dependency = module_graph.dependency_by_id(&conn.dependency_id);
          let Some(export_usages) = dependency_export_usage(
            dependency.as_ref(),
            module_graph,
            module_graph_cache,
            exports_info_artifact,
          ) else {
            return vec![];
          };
          if export_usages.is_empty() {
            return vec![];
          }

          let dependency_id = conn.dependency_id;
          let origin_module_identifier = conn.original_module_identifier.unwrap_or(*module_id);
          let target_module_identifier = *conn.module_identifier();

          export_usages
            .into_iter()
            .map(
              |(origin_export, target_export)| RsdoctorExportUsageDependency {
                dependency_id,
                origin_module_identifier,
                target_module_identifier,
                origin_export,
                target_export,
              },
            )
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>()
}

#[inline(never)]
fn is_origin_export_used(
  candidate: &RsdoctorExportUsageDependency,
  exports_info_artifact: &ExportsInfoArtifact,
) -> bool {
  let Some(origin_export) = &candidate.origin_export else {
    return true;
  };
  let origin_export = origin_export
    .iter()
    .map(|name| Atom::from(name.as_str()))
    .collect::<Vec<_>>();
  let origin_exports_info =
    exports_info_artifact.get_exports_info_data(&candidate.origin_module_identifier);
  origin_exports_info.get_used(exports_info_artifact, &origin_export, None) != UsageState::Unused
}

fn dependency_has_impure_deferred_pure_checks(
  dependency: &dyn Dependency,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> bool {
  dependency
    .downcast_ref::<ESMImportSpecifierDependency>()
    .and_then(|dependency| dependency.used_by_exports())
    .or_else(|| {
      dependency
        .downcast_ref::<URLDependency>()
        .and_then(|dependency| dependency.used_by_exports())
    })
    .is_some_and(|used_by_exports| {
      has_impure_deferred_pure_checks(module_graph, exports_info_artifact, used_by_exports)
    })
}

pub fn collect_active_export_usage_dependencies(
  candidates: &[RsdoctorExportUsageDependency],
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  side_effects_state_artifact: &SideEffectsStateArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<RsdoctorExportUsageDependency> {
  candidates
    .iter()
    .filter(|candidate| {
      let dependency = module_graph.dependency_by_id(&candidate.dependency_id);
      if !dependency_has_impure_deferred_pure_checks(
        dependency.as_ref(),
        module_graph,
        exports_info_artifact,
      ) && !is_origin_export_used(candidate, exports_info_artifact)
      {
        return false;
      }
      module_graph
        .connection_by_dependency_id(&candidate.dependency_id)
        .is_some_and(|connection| {
          connection.is_active(
            module_graph,
            None,
            module_graph_cache,
            side_effects_state_artifact,
            exports_info_artifact,
          )
        })
    })
    .cloned()
    .collect::<Vec<_>>()
}

pub fn collect_export_usage_edges(
  dependencies: Vec<RsdoctorExportUsageDependency>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
) -> Vec<RsdoctorExportUsageEdge> {
  dependencies
    .into_iter()
    .filter_map(|dependency| {
      let origin_module = *module_ukeys.get(&dependency.origin_module_identifier)?;
      let target_module = *module_ukeys.get(&dependency.target_module_identifier)?;

      Some(RsdoctorExportUsageEdge {
        origin_module,
        origin_export: dependency.origin_export,
        target_module,
        target_export: dependency.target_export,
      })
    })
    .collect::<Vec<_>>()
}

pub fn collect_module_ids(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
  module_ids: &ModuleIdsArtifact,
) -> Vec<RsdoctorModuleId> {
  modules
    .keys()
    .par_bridge()
    .filter_map(|module_id| {
      let render_id = ChunkGraph::get_module_id(module_ids, *module_id).map(|s| s.to_string())?;
      let module_ukey = module_ukeys.get(module_id)?;
      Some(RsdoctorModuleId {
        module: *module_ukey,
        render_id,
      })
    })
    .collect::<Vec<_>>()
}

pub fn collect_module_side_effects_locations(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
  module_graph: &ModuleGraph,
) -> IdentifierMap<Vec<RsdoctorSideEffectLocation>> {
  modules
    .par_iter()
    .filter_map(|(module_id, module)| {
      let bailout_reasons = module_graph.get_optimization_bailout(module_id);
      let module_ukey = module_ukeys.get(module_id)?;
      let request = if let Some(normal_module) = module.as_normal_module() {
        normal_module.request().to_string()
      } else {
        module.identifier().to_string()
      };

      let side_effect_locations: Vec<RsdoctorSideEffectLocation> = bailout_reasons
        .iter()
        .filter_map(|item| match item {
          OptimizationBailoutItem::SideEffects { node_type, loc, .. } => {
            Some(RsdoctorSideEffectLocation {
              location: loc.clone(),
              node_type: node_type.clone(),
              module: *module_ukey,
              request: request.clone(),
            })
          }
          _ => None,
        })
        .collect();
      if side_effect_locations.is_empty() {
        None
      } else {
        Some((*module_id, side_effect_locations))
      }
    })
    .collect::<IdentifierMap<Vec<RsdoctorSideEffectLocation>>>()
}

pub fn collect_connections_only_imports(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &IdentifierMap<ModuleUkey>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  side_effects_state_artifact: &SideEffectsStateArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  module_ukey_to_info: &HashMap<ModuleUkey, (String, bool)>,
) -> Vec<RsdoctorConnectionsOnlyImport> {
  let connections = modules
    .par_iter()
    .flat_map(|(module_id, _)| {
      if module_ukeys.get(module_id).is_none() {
        return vec![];
      }

      module_graph
        .get_outgoing_connections(module_id)
        .filter_map(|conn| {
          let dep = module_graph.dependency_by_id(&conn.dependency_id);
          let dependency_type = dep.dependency_type().to_string();
          let user_request = dep
            .as_module_dependency()
            .map(|d| d.user_request().to_string())
            .unwrap_or_default();

          let origin_module = conn
            .original_module_identifier
            .as_ref()
            .and_then(|id| module_ukeys.get(id).copied());
          let resolved_module = module_ukeys.get(&conn.resolved_module).copied()?;

          let active = conn.is_active(
            module_graph,
            None,
            module_graph_cache,
            side_effects_state_artifact,
            exports_info_artifact,
          );

          Some((
            resolved_module,
            RsdoctorConnectionsOnlyImportConnection {
              origin_module,
              dependency_type,
              user_request,
              active,
            },
          ))
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

  let mut grouped: HashMap<ModuleUkey, Vec<RsdoctorConnectionsOnlyImportConnection>> =
    HashMap::default();

  for (resolved_module, connection) in connections {
    grouped.entry(resolved_module).or_default().push(connection);
  }

  grouped
    .into_iter()
    .filter_map(|(module_ukey, connections)| {
      let (path, is_entry) = module_ukey_to_info.get(&module_ukey)?;

      // Entry modules are expected to be referenced directly — skip them.
      if *is_entry {
        return None;
      }

      // Check if there is exactly one active connection
      let active_connections: Vec<_> = connections.into_iter().filter(|c| c.active).collect();
      if active_connections.len() != 1 {
        return None;
      }

      Some(RsdoctorConnectionsOnlyImport {
        module_ukey,
        module_path: path.clone(),
        connections: active_connections,
      })
    })
    .collect::<Vec<_>>()
}
