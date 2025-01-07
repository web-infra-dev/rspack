use std::sync::{atomic::AtomicUsize, Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{Identifier, IdentifierMap};
use rspack_core::{
  rspack_sources::MapOptions, BoxModule, ChunkGraph, Compilation, Context, DependencyId,
  ModuleGraph,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  ChunkUkey, ModuleKind, ModuleUkey, RsdoctorDependency, RsdoctorModule, RsdoctorModuleSource,
};

pub fn collect_modules(
  modules: &IdentifierMap<&BoxModule>,
  module_graph: &ModuleGraph,
  chunk_graph: &ChunkGraph,
  context: &Context,
) -> HashMap<Identifier, RsdoctorModule> {
  let module_ukey_counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

  modules
    .par_iter()
    .map(|(module_id, module)| {
      let ukey = module_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      let depth = module_graph.get_depth(module_id);
      let path = if let Some(nfc) = module.name_for_condition() {
        nfc.to_string()
      } else {
        module.readable_identifier(&context).to_string()
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
          chunks,
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_concatenated_modules(
  modules: &IdentifierMap<&BoxModule>,
  rsd_modules: &HashMap<Identifier, RsdoctorModule>,
) -> HashMap<Identifier, HashSet<ModuleUkey>> {
  modules
    .par_iter()
    .map(|(module_id, module)| {
      (
        module_id.to_owned(),
        module
          .as_concatenated_module()
          .map(|concatenated_module| {
            concatenated_module
              .get_modules()
              .iter()
              .filter_map(|m| rsd_modules.get(&m.id).map(|m| m.ukey))
              .collect::<HashSet<_>>()
          })
          .unwrap_or_default(),
      )
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_module_dependencies(
  modules: &IdentifierMap<&BoxModule>,
  rsd_modules: &HashMap<Identifier, RsdoctorModule>,
  module_graph: &ModuleGraph,
) -> HashMap<Identifier, HashMap<Identifier, (DependencyId, RsdoctorDependency)>> {
  let dependency_ukey_counter = Arc::new(AtomicUsize::new(0));

  modules
    .par_iter()
    .filter_map(|(module_id, _)| {
      let Some(rsd_module_ukey) = rsd_modules.get(module_id).map(|m| m.ukey) else {
        return None;
      };
      let dependencies = module_graph
        .get_outgoing_connections(module_id)
        .filter_map(|conn| {
          let Some(dep) = module_graph.dependency_by_id(&conn.dependency_id) else {
            return None;
          };
          if let (Some(dep), Some(dep_module)) = (
            dep.as_module_dependency(),
            module_graph
              .module_identifier_by_dependency_id(&conn.dependency_id)
              .and_then(|mid| rsd_modules.get(mid)),
          ) {
            let dep_ukey =
              dependency_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some((
              dep_module.identifier,
              (
                conn.dependency_id,
                RsdoctorDependency {
                  ukey: dep_ukey,
                  kind: *dep.dependency_type(),
                  request: dep.user_request().into(),
                  module: rsd_module_ukey,
                  dependency: dep_module.ukey,
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
  compilation: &Compilation,
) -> HashMap<Identifier, RsdoctorModuleSource> {
  modules
    .par_iter()
    .filter_map(|(module_id, module)| {
      let source = module.original_source();
      let size = module.size(None, Some(compilation)) as usize;
      Some((
        module_id.to_owned(),
        RsdoctorModuleSource {
          source_size: size,
          transform_size: size,
          source: source.map(|s| s.source().to_string()),
          source_map: source
            .and_then(|s| s.map(&MapOptions::default()))
            .and_then(|m| m.to_json().ok()),
        },
      ))
    })
    .collect::<HashMap<_, _>>()
}
