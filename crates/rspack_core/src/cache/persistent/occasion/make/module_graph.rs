use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  utils::OwnedOrRef,
  with::{AsOption, AsOwned, AsTuple2, AsVec},
  SerializeError,
};
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;

use super::Storage;
use crate::{
  cache::persistent::cacheable_context::CacheableContext, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, DependencyId, DependencyParents,
  ExportInfoData, ExportsInfoData, ModuleGraph, ModuleGraphConnection, ModuleGraphModule,
  ModuleGraphPartial, RayonConsumer,
};

const SCOPE: &str = "occasion_make_module_graph";

/// The value struct of current storage scope
#[cacheable]
struct Node<'a> {
  #[cacheable(with=AsOwned)]
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  #[cacheable(with=AsOwned)]
  pub module: OwnedOrRef<'a, BoxModule>,
  #[cacheable(with=AsVec<AsTuple2<AsOwned, AsOption<AsOwned>>>)]
  pub dependencies: Vec<(
    OwnedOrRef<'a, BoxDependency>,
    Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
  )>,
  #[cacheable(with=AsVec<AsOwned>)]
  pub connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
  #[cacheable(with=AsVec<AsOwned>)]
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::save", skip_all)]
pub fn save_module_graph(
  partial: &ModuleGraphPartial,
  revoked_modules: &IdentifierSet,
  built_modules: &IdentifierSet,
  storage: &Arc<dyn Storage>,
  context: &CacheableContext,
) {
  let mg = ModuleGraph::new(vec![partial], None);
  for identifier in revoked_modules {
    storage.remove(SCOPE, identifier.as_bytes());
  }

  // save module_graph
  let nodes = built_modules
    .par_iter()
    .filter_map(|identifier| {
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
      let node = Node {
        mgm: mgm.into(),
        module: module.into(),
        dependencies,
        connections,
        blocks,
      };
      match to_bytes(&node, context) {
        Ok(bytes) => Some((identifier.as_bytes().to_vec(), bytes)),
        Err(err) => {
          if matches!(err, SerializeError::UnsupportedField) {
            tracing::warn!("to bytes failed {:?}", err);
            None
          } else {
            panic!("unexpected module graph serialize failed")
          }
        }
      }
    })
    .collect::<Vec<_>>();

  let saved = nodes.len();
  tracing::debug!("save info {}/{}", saved, built_modules.len());

  for (id, bytes) in nodes {
    storage.set(SCOPE, id, bytes)
  }
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::recovery", skip_all)]
pub async fn recovery_module_graph(
  storage: &Arc<dyn Storage>,
  context: &CacheableContext,
) -> Result<(ModuleGraphPartial, HashSet<DependencyId>)> {
  let mut need_check_dep = vec![];
  let mut partial = ModuleGraphPartial::default();
  let mut mg = ModuleGraph::new(vec![], Some(&mut partial));
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
      // recovery exports/export info
      let other_exports_info = ExportInfoData::new(None, None);
      let side_effects_only_info = ExportInfoData::new(Some("*side effects only*".into()), None);
      let exports_info = ExportsInfoData::new(other_exports_info.id(), side_effects_only_info.id());
      mgm.exports = exports_info.id();
      mg.set_exports_info(exports_info.id(), exports_info);
      mg.set_export_info(side_effects_only_info.id(), side_effects_only_info);
      mg.set_export_info(other_exports_info.id(), other_exports_info);

      mg.add_module_graph_module(mgm);
      mg.add_module(module);
    });
  let mut force_build_dependencies = HashSet::default();
  // recovery incoming connections
  for (dep_id, module_identifier) in need_check_dep {
    if let Some(mgm) = mg.module_graph_module_by_identifier_mut(&module_identifier) {
      mgm.add_incoming_connection(dep_id);
    } else {
      force_build_dependencies.insert(dep_id);
    }
  }

  tracing::debug!("recovery {} module", mg.modules().len());
  tracing::debug!("recovery failed {} deps", force_build_dependencies.len());
  Ok((partial, force_build_dependencies))
}
