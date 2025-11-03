use std::sync::{Arc, atomic::AtomicI32};

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{Identifier, IdentifierMap};
use rspack_core::{
  BoxModule, ChunkGraph, Compilation, Context, DependencyId, DependencyType, Module, ModuleGraph,
  ModuleIdsArtifact,
  rspack_sources::{MapOptions, ObjectPool},
};
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use thread_local::ThreadLocal;

use crate::{
  ChunkUkey, ModuleKind, ModuleUkey, RsdoctorDependency, RsdoctorModule, RsdoctorModuleId,
  RsdoctorModuleOriginalSource,
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
          layer: module.get_layer().map(|layer| layer.to_string()),
          dependencies: HashSet::default(),
          imported: HashSet::default(),
          modules: HashSet::default(),
          belong_modules: HashSet::default(),
          chunks,
          issuer_path: None,
          bailout_reason: HashSet::default(),
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
      let module_ukey = module_ukeys.get(module_id)?;
      let resource = module.resource_resolved_data().resource().to_owned();
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
            .dependency_by_id(&conn.dependency_id)?
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
