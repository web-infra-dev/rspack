use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::{SerializeError, cacheable, from_bytes, to_bytes, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;

use super::{
  Storage,
  alternatives::{TempDependency, TempModule},
};
use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, Dependency,
  DependencyId, DependencyParents, ExportsInfoData, ModuleGraph, ModuleGraphConnection,
  ModuleGraphModule, ModuleGraphPartial, ModuleIdentifier, RayonConsumer,
  cache::persistent::cacheable_context::CacheableContext,
  compilation::make::{LazyDependencies, ModuleToLazyMake},
};

const SCOPE: &str = "occasion_make_module_graph";

/// The value struct of current storage scope
#[cacheable]
struct Node<'a> {
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  pub module: OwnedOrRef<'a, BoxModule>,
  pub dependencies: Vec<(
    OwnedOrRef<'a, BoxDependency>,
    Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
  )>,
  pub connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
  pub lazy_info: Option<OwnedOrRef<'a, LazyDependencies>>,
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::save", skip_all)]
pub fn save_module_graph(
  partial: &ModuleGraphPartial,
  module_to_lazy_make: &ModuleToLazyMake,
  revoked_modules: &IdentifierSet,
  need_update_modules: &IdentifierSet,
  storage: &Arc<dyn Storage>,
  context: &CacheableContext,
) {
  let mg = ModuleGraph::new([Some(partial), None], None);
  for identifier in revoked_modules {
    storage.remove(SCOPE, identifier.as_bytes());
  }

  // save module_graph
  let nodes = need_update_modules
    .par_iter()
    .map(|identifier| {
      let mgm = mg
        .module_graph_module_by_identifier(identifier)
        .expect("should have mgm");
      let module = mg
        .module_by_identifier(identifier)
        .expect("should have module");
      let blocks = module
        .get_blocks()
        .par_iter()
        .map(|block_id| mg.block_by_id(block_id).expect("should have block").into())
        .collect::<Vec<_>>();
      let dependencies = mgm
        .all_dependencies
        .par_iter()
        .map(|dep_id| {
          (
            mg.dependency_by_id(dep_id)
              .expect("should have dependency")
              .into(),
            mg.get_parent_block(dep_id).map(Into::into),
          )
        })
        .collect::<Vec<_>>();
      let connections = mgm
        .outgoing_connections()
        .par_iter()
        .map(|dep_id| {
          mg.connection_by_dependency_id(dep_id)
            .expect("should have connection")
            .into()
        })
        .collect::<Vec<_>>();
      let lazy_info = module_to_lazy_make
        .get_lazy_dependencies(identifier)
        .map(|lazy_deps| lazy_deps.into());
      let mut node = Node {
        mgm: mgm.into(),
        module: module.into(),
        dependencies,
        connections,
        blocks,
        lazy_info,
      };
      match to_bytes(&node, context) {
        Ok(bytes) => (identifier.as_bytes().to_vec(), bytes),
        Err(err @ SerializeError::UnsupportedField) => {
          tracing::warn!("to bytes failed {:?}", err);
          // try use alternatives
          node.module = TempModule::transform_from(node.module);
          node.dependencies = node
            .dependencies
            .into_iter()
            .map(|(dep, _)| (TempDependency::transform_from(dep), None))
            .collect();
          node.blocks = vec![];
          if let Ok(bytes) = to_bytes(&node, context) {
            (identifier.as_bytes().to_vec(), bytes)
          } else {
            panic!("alternatives serialize failed")
          }
        }
        Err(_) => {
          panic!("unexpected module graph serialize failed")
        }
      }
    })
    .collect::<Vec<_>>();

  tracing::debug!("save {} modules", nodes.len());

  for (id, bytes) in nodes {
    storage.set(SCOPE, id, bytes)
  }
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::recovery", skip_all)]
pub async fn recovery_module_graph(
  storage: &Arc<dyn Storage>,
  context: &CacheableContext,
) -> Result<(ModuleGraphPartial, ModuleToLazyMake, HashSet<DependencyId>)> {
  let mut need_check_dep = vec![];
  let mut partial = ModuleGraphPartial::default();
  let mut module_to_lazy_make = ModuleToLazyMake::default();
  let mut mg = ModuleGraph::new([None, None], Some(&mut partial));
  storage
    .load(SCOPE)
    .await?
    .into_par_iter()
    .map(|(_, v)| {
      from_bytes::<Node, CacheableContext>(&v, context)
        .expect("unexpected module graph deserialize failed")
    })
    .with_max_len(1)
    .consume(|node| {
      let mut mgm = node.mgm.into_owned();
      let module = node.module.into_owned();
      for (index_in_block, (dep, parent_block)) in node.dependencies.into_iter().enumerate() {
        let dep = dep.into_owned();
        mg.set_parents(
          *dep.id(),
          DependencyParents {
            block: parent_block.map(|b| b.into_owned()),
            module: module.identifier(),
            index_in_block,
          },
        );
        mg.add_dependency(dep);
      }
      for con in node.connections {
        let con = con.into_owned();
        need_check_dep.push((con.dependency_id, *con.module_identifier()));
        mg.cache_recovery_connection(con);
      }
      for block in node.blocks {
        let block = block.into_owned();
        mg.add_block(Box::new(block));
      }
      if let Some(lazy_info) = node.lazy_info {
        module_to_lazy_make
          .update_module_lazy_dependencies(module.identifier(), Some(lazy_info.into_owned()));
      }
      // recovery exports/export info
      let exports_info = ExportsInfoData::default();
      mgm.exports = exports_info.id();
      mg.set_exports_info(exports_info.id(), exports_info);

      mg.add_module_graph_module(mgm);
      mg.add_module(module);
    });
  // recovery incoming connections
  for (dep_id, module_identifier) in need_check_dep {
    let mgm = mg
      .module_graph_module_by_identifier_mut(&module_identifier)
      .expect("should mgm exist");
    mgm.add_incoming_connection(dep_id);
  }

  // recovery entry
  let mut entry_module: Vec<ModuleIdentifier> = vec![];
  for (_, mgm) in mg.module_graph_modules() {
    if mgm.issuer().identifier().is_none() {
      entry_module.push(mgm.module_identifier);
    };
  }
  let mut entry_dependencies: HashSet<DependencyId> = Default::default();
  for mid in entry_module {
    let dep = TempDependency::default();
    let connection = ModuleGraphConnection::new(*dep.id(), None, mid, false);
    entry_dependencies.insert(*dep.id());
    mg.add_dependency(Box::new(dep));
    mg.cache_recovery_connection(connection);
  }

  tracing::debug!("recovery {} module", mg.modules().len());
  Ok((partial, module_to_lazy_make, entry_dependencies))
}
