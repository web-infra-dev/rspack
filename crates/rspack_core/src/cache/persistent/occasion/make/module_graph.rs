use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  with::{AsOption, AsTuple2, AsVec, Inline},
  DeserializeError, SerializeError,
};
use rspack_collections::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;

use super::Storage;
use crate::{
  cache::persistent::cacheable_context::CacheableContext, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildDependency, DependencyParents,
  ExportInfoData, ExportsInfoData, ModuleGraph, ModuleGraphConnection, ModuleGraphModule,
  ModuleGraphPartial,
};

const SCOPE: &str = "occasion_make_module_graph";

/// The value struct of current storage scope
#[cacheable]
struct Node {
  pub mgm: ModuleGraphModule,
  pub module: BoxModule,
  // (dependency, parent_block)
  // TODO remove parent block info after connection contains it
  pub dependencies: Vec<(BoxDependency, Option<AsyncDependenciesBlockIdentifier>)>,
  pub connections: Vec<ModuleGraphConnection>,
  pub blocks: Vec<AsyncDependenciesBlock>,
}

#[cacheable(as=Node)]
struct NodeRef<'a> {
  #[cacheable(with=Inline)]
  pub mgm: &'a ModuleGraphModule,
  #[cacheable(with=Inline)]
  pub module: &'a BoxModule,
  #[cacheable(with=AsVec<AsTuple2<Inline, AsOption<Inline>>>)]
  pub dependencies: Vec<(
    &'a BoxDependency,
    Option<&'a AsyncDependenciesBlockIdentifier>,
  )>,
  #[cacheable(with=AsVec<Inline>)]
  pub connections: Vec<&'a ModuleGraphConnection>,
  #[cacheable(with=AsVec<Inline>)]
  pub blocks: Vec<&'a AsyncDependenciesBlock>,
}

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
        .map(|block_id| mg.block_by_id(block_id).expect("should have block"))
        .collect::<Vec<_>>();
      let dependencies = mgm
        .all_dependencies
        .par_iter()
        .map(|dep_id| {
          (
            mg.dependency_by_id(dep_id).expect("should have dependency"),
            mg.get_parent_block(dep_id),
          )
        })
        .collect::<Vec<_>>();
      let connections = mgm
        .outgoing_connections()
        .par_iter()
        .map(|dep_id| {
          mg.connection_by_dependency_id(dep_id)
            .expect("should have connection")
        })
        .collect::<Vec<_>>();
      let node = NodeRef {
        mgm,
        module,
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
  tracing::info!("save info {}/{}", saved, built_modules.len());

  for (id, bytes) in nodes {
    storage.set(SCOPE, id, bytes)
  }
}

pub async fn recovery_module_graph(
  storage: &Arc<dyn Storage>,
  context: &CacheableContext,
) -> Result<(ModuleGraphPartial, HashSet<BuildDependency>), DeserializeError> {
  let mut need_check_dep = vec![];
  let mut partial = ModuleGraphPartial::default();
  let mut mg = ModuleGraph::new(vec![], Some(&mut partial));
  for (_, v) in storage.load(SCOPE).await.unwrap_or_default() {
    let mut node: Node =
      from_bytes(&v, context).expect("unexpected module graph deserialize failed");
    for (dep, parent_block) in node.dependencies {
      mg.set_parents(
        *dep.id(),
        DependencyParents {
          block: parent_block,
          module: node.module.identifier(),
        },
      );
      mg.add_dependency(dep);
    }
    for con in node.connections {
      need_check_dep.push((con.dependency_id, *con.module_identifier()));
      mg.cache_recovery_connection(con);
    }
    for block in node.blocks {
      mg.add_block(Box::new(block));
    }
    // recovery exports/export info
    let other_exports_info = ExportInfoData::new(None, None);
    let side_effects_only_info = ExportInfoData::new(Some("*side effects only*".into()), None);
    let exports_info = ExportsInfoData::new(other_exports_info.id(), side_effects_only_info.id());
    node.mgm.exports = exports_info.id();
    mg.set_exports_info(exports_info.id(), exports_info);
    mg.set_export_info(side_effects_only_info.id(), side_effects_only_info);
    mg.set_export_info(other_exports_info.id(), other_exports_info);

    mg.add_module_graph_module(node.mgm);
    mg.add_module(node.module);
  }
  // recovery incoming connections
  for (con_id, module_identifier) in &need_check_dep {
    if let Some(mgm) = mg.module_graph_module_by_identifier_mut(module_identifier) {
      mgm.add_incoming_connection(*con_id);
    }
  }

  println!("recovery {} module", mg.modules().len());
  let make_failed_dependencies = need_check_dep
    .iter()
    .filter_map(|(dep_id, module_identifier)| {
      let module_exist = mg.module_by_identifier(module_identifier).is_some();
      if module_exist {
        None
      } else {
        mg.revoke_connection(dep_id, false)
      }
    })
    .collect::<HashSet<_>>();
  println!("recovery failed {} deps", make_failed_dependencies.len());
  Ok((partial, make_failed_dependencies))
}
