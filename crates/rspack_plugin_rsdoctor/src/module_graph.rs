use std::sync::{atomic::AtomicI32, Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{Identifier, IdentifierMap};
use rspack_core::{
  rspack_sources::MapOptions, BoxModule, ChunkGraph, Compilation, Context, DependencyId,
  ModuleGraph, ModuleIdsArtifact,
};
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  ChunkUkey, ModuleKind, ModuleUkey, RsdoctorDependency, RsdoctorModule, RsdoctorModuleId,
  RsdoctorModuleSource,
};

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
      let path = if let Some(nfc) = module.name_for_condition() {
        nfc.to_string()
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
          layer: module.get_layer().map(|layer| layer.to_string()),
          dependencies: HashSet::default(),
          imported: HashSet::default(),
          modules: HashSet::default(),
          belong_modules: HashSet::default(),
          chunks,
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
    .map(|(parent, children)| {
      children
        .iter()
        .map(|child| (*child, *parent))
        .collect::<HashSet<_>>()
    })
    .flatten()
    .fold(HashMap::default(), |mut acc, (child, parent)| {
      acc
        .entry(child)
        .or_insert_with(HashSet::default)
        .insert(parent);
      acc
    });

  (children_map, parent_map)
}

pub fn collect_module_dependencies(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &FxDashMap<Identifier, ModuleUkey>,
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
          let dep = module_graph.dependency_by_id(&conn.dependency_id)?;
          if let (Some(dep), Some(dep_module)) = (
            dep.as_module_dependency(),
            module_graph
              .module_identifier_by_dependency_id(&conn.dependency_id)
              .and_then(|mid| module_ukeys.get(mid).map(|ukey| (*mid, *ukey))),
          ) {
            let dep_ukey =
              dependency_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some((
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
            ));
          }
          None
        })
        .collect::<HashMap<_, _>>();

      Some((module_id.to_owned(), dependencies))
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_module_sources(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &FxDashMap<Identifier, ModuleUkey>,
  compilation: &Compilation,
) -> Vec<RsdoctorModuleSource> {
  modules
    .par_iter()
    .filter_map(|(module_id, module)| {
      let source = module.original_source();
      let size = module.size(None, Some(compilation)) as i32;
      let ukey = module_ukeys.get(module_id)?;
      Some(RsdoctorModuleSource {
        module: *ukey,
        source_size: size,
        transform_size: size,
        source: source.map(|s| s.source().to_string()),
        source_map: source
          .and_then(|s| s.map(&MapOptions::default()))
          .and_then(|m| m.to_json().ok()),
      })
    })
    .collect::<Vec<_>>()
}

pub fn collect_module_ids(
  modules: &IdentifierMap<&BoxModule>,
  module_ukeys: &FxDashMap<Identifier, ModuleUkey>,
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
