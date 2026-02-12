use std::sync::{Arc, atomic::AtomicI32};

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{Identifiable, Identifier, IdentifierMap};
use rspack_core::{
  BoxModule, ChunkGraph, Compilation, Context, DependencyId, DependencyType, Module, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdsArtifact, ModuleType, PrefetchExportsInfoMode, UsageState,
  rspack_sources::{MapOptions, ObjectPool},
};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_json::create_object_for_exports_info;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use thread_local::ThreadLocal;

use crate::{
  ChunkUkey, ModuleKind, ModuleUkey, RsdoctorConnection, RsdoctorDependency,
  RsdoctorJsonModuleSizes, RsdoctorModule, RsdoctorModuleId, RsdoctorModuleOriginalSource,
  RsdoctorSideEffectLocation,
};

pub fn collect_json_module_sizes(
  modules: &IdentifierMap<&BoxModule>,
  module_graph: &ModuleGraph,
) -> RsdoctorJsonModuleSizes {
  let mut json_sizes: RsdoctorJsonModuleSizes = RsdoctorJsonModuleSizes::default();

  for (module_id, module) in modules.iter() {
    if module.module_type() != &ModuleType::Json {
      continue;
    }

    let Some(json_data) = module.build_info().json_data.as_ref() else {
      continue;
    };

    let exports_info =
      module_graph.get_prefetched_exports_info(module_id, PrefetchExportsInfoMode::Default);

    let final_json = match json_data {
      json::JsonValue::Object(_) | json::JsonValue::Array(_) => {
        let needs_tree_shaking = exports_info.other_exports_info().get_used(None)
          == UsageState::Unused
          || exports_info.exports().any(|(_, info)| {
            let used = info.get_used(None);
            used == UsageState::Unused || used == UsageState::OnlyPropertiesUsed
          });

        if needs_tree_shaking {
          create_object_for_exports_info(json_data.clone(), &exports_info, None, module_graph)
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
) -> HashMap<Identifier, RsdoctorModule> {
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
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_concatenated_modules(
  modules: &IdentifierMap<&BoxModule>,
) -> (
  HashMap<Identifier, HashSet<Identifier>>,
  HashMap<Identifier, HashSet<Identifier>>,
) {
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
          .collect::<HashSet<_>>(),
      ))
    })
    .collect::<HashMap<_, _>>();

  let parent_map = children_map
    .iter()
    .flat_map(|(parent, children)| {
      children
        .iter()
        .map(|child| (*child, *parent))
        .collect::<HashSet<_>>()
    })
    .fold(
      HashMap::default(),
      |mut acc: HashMap<Identifier, HashSet<Identifier>>, (child, parent)| {
        acc.entry(child).or_default().insert(parent);
        acc
      },
    );

  (children_map, parent_map)
}

pub fn collect_module_original_sources(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &HashMap<Identifier, ModuleUkey>,
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
  module_ukeys: &HashMap<Identifier, ModuleUkey>,
  module_graph: &ModuleGraph,
) -> HashMap<Identifier, HashMap<Identifier, (DependencyId, RsdoctorDependency)>> {
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
        .collect::<HashMap<_, _>>();

      Some((module_id.to_owned(), dependencies))
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_module_ids(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &HashMap<Identifier, ModuleUkey>,
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

pub fn collect_module_connections(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &HashMap<Identifier, ModuleUkey>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
) -> Vec<RsdoctorConnection> {
  let connection_ukey_counter = Arc::new(AtomicI32::new(0));

  modules
    .par_iter()
    .flat_map(|(module_id, _)| {
      let Some(module_ukey) = module_ukeys.get(module_id) else {
        return vec![];
      };

      module_graph
        .get_incoming_connections(module_id)
        .filter_map(|conn| {
          let dep = module_graph.dependency_by_id(&conn.dependency_id);

          // Get dependency type and user_request
          let (dep_type, user_request) = if let Some(d) = dep.as_module_dependency() {
            (
              d.dependency_type().as_str().to_string(),
              d.user_request().to_string(),
            )
          } else if let Some(d) = dep.as_context_dependency() {
            (
              d.dependency_type().as_str().to_string(),
              d.request().to_string(),
            )
          } else {
            (dep.dependency_type().as_str().to_string(), String::new())
          };

          // Get loc
          let loc = dep.loc().map(|l| l.to_string());

          // Calculate active state
          let active = conn.is_active(module_graph, None, module_graph_cache);

          // Get origin module ukey
          let origin_module_ukey = conn
            .original_module_identifier
            .and_then(|id| module_ukeys.get(&id).copied());

          // Get resolved module ukey
          let resolved_module_ukey = module_ukeys
            .get(&conn.resolved_module)
            .copied()
            .unwrap_or(*module_ukey);

          let ukey = connection_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

          Some(RsdoctorConnection {
            ukey,
            dependency_id: conn.dependency_id.as_u32().to_string(),
            module: *module_ukey,
            origin_module: origin_module_ukey,
            resolved_module: resolved_module_ukey,
            dependency_type: dep_type,
            user_request,
            loc,
            active,
          })
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>()
}

pub fn collect_module_side_effects_locations(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &HashMap<Identifier, ModuleUkey>,
  module_graph: &ModuleGraph,
) -> HashMap<Identifier, Vec<RsdoctorSideEffectLocation>> {
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
        .filter_map(|reason| {
          // Parse bailout_reason string format:
          // "{node_type} with side_effects in source code at {file}:{location}"
          if !reason.contains("side_effects") {
            return None;
          }

          // Extract node type and location
          let parts: Vec<&str> = reason
            .split(" with side_effects in source code at ")
            .collect();
          if parts.len() != 2 {
            return None;
          }

          let node_type = parts[0].to_string();
          let location_part = parts[1];

          // Format: "module_identifier:line:column"
          let location = if let Some(colon_pos) = location_part.rfind(':') {
            let before_last_colon = &location_part[..colon_pos];
            if let Some(second_colon_pos) = before_last_colon.rfind(':') {
              location_part[second_colon_pos + 1..].to_string()
            } else {
              location_part.to_string()
            }
          } else {
            location_part.to_string()
          };

          Some(RsdoctorSideEffectLocation {
            location,
            node_type,
            module: *module_ukey,
            request: request.clone(),
          })
        })
        .collect();
      if side_effect_locations.is_empty() {
        None
      } else {
        Some((*module_id, side_effect_locations))
      }
    })
    .collect::<HashMap<_, _>>()
}
