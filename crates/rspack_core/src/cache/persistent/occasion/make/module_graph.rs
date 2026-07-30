use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_tasks::get_current_dependency_id;
use rustc_hash::FxHashSet;

use super::alternatives::{TempDependency, TempModule};
use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule,
  DependenciesBlock, Dependency, DependencyId, DependencyParents, ModuleGraph,
  ModuleGraphConnection, ModuleGraphModule, ModuleIdentifier, RayonConsumer,
  cache::persistent::{codec::CacheCodec, storage::Storage},
  compilation::build_module_graph::{LazyDependencies, ModuleToLazyMake},
};

pub const SCOPE: &str = "occasion_make_module_graph";

/// The value struct of current storage scope
#[cacheable]
struct Node<'a> {
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  pub module: OwnedOrRef<'a, BoxModule>,
  pub dependencies: Vec<(
    OwnedOrRef<'a, BoxDependency>,
    Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
    usize,
  )>,
  pub connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
  pub lazy_info: Option<OwnedOrRef<'a, LazyDependencies>>,
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::save", skip_all)]
pub fn save_module_graph(
  mg: &ModuleGraph,
  module_to_lazy_make: &ModuleToLazyMake,
  removed_modules: &IdentifierSet,
  need_update_modules: &IdentifierSet,
  storage: &mut dyn Storage,
  codec: &CacheCodec,
) {
  for identifier in removed_modules {
    storage.remove(SCOPE, identifier.as_bytes());
  }

  // save module_graph
  let saved_count = AtomicUsize::new(0);
  need_update_modules
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
        .all_dependencies()
        .par_iter()
        .map(|dep_id| {
          (
            mg.dependency_by_id(dep_id).into(),
            mg.get_parent_block(dep_id).map(Into::into),
            mg.get_parent_block_index(dep_id)
              .expect("dependency should have a parent index"),
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
      match codec.encode(&node) {
        Ok(bytes) => (identifier.as_bytes().to_vec(), bytes),
        Err(err) if err.to_string().contains("unsupported field") => {
          tracing::warn!("to bytes failed {:?}", err);
          // try use alternatives
          node.module = TempModule::transform_from(node.module);
          node.dependencies = node
            .dependencies
            .into_iter()
            .map(|(dep, _, index_in_block)| {
              (TempDependency::transform_from(dep), None, index_in_block)
            })
            .collect();
          node.blocks = vec![];
          if let Ok(bytes) = codec.encode(&node) {
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
    .consume(|(id, bytes)| {
      storage.set(SCOPE, id, bytes);
      saved_count.fetch_add(1, Ordering::Relaxed);
    });

  tracing::debug!("save {} modules", saved_count.load(Ordering::Relaxed));
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::recovery", skip_all)]
pub async fn recovery_module_graph(
  storage: &dyn Storage,
  codec: &CacheCodec,
) -> Result<(ModuleGraph, ModuleToLazyMake, FxHashSet<DependencyId>)> {
  let mut recovered_connection_ids = vec![];
  let mut mg = ModuleGraph::default();
  let mut module_to_lazy_make = ModuleToLazyMake::default();
  let mut recovery_error = None;
  let max_dependency_id = get_current_dependency_id();
  storage
    .load(SCOPE)
    .await?
    .into_par_iter()
    .map(|(_, v)| codec.decode::<Node>(&v))
    .with_max_len(1)
    .consume(|result| {
      let node = match result {
        Ok(node) if recovery_error.is_none() => node,
        Ok(_) => return,
        Err(err) => {
          recovery_error = Some(err);
          return;
        }
      };
      let mgm = node.mgm.into_owned();
      let module = node.module.into_owned();
      if mgm.module_identifier != module.identifier() {
        recovery_error = Some(rspack_error::error!(format!(
          "persistent cache module graph entry has mismatched module identifiers: {} and {}",
          mgm.module_identifier,
          module.identifier()
        )));
        return;
      }
      if mg
        .module_graph_module_by_identifier(&mgm.module_identifier)
        .is_some()
        || mg.module_by_identifier(&mgm.module_identifier).is_some()
      {
        recovery_error = Some(rspack_error::error!(format!(
          "persistent cache module graph contains duplicate module {}",
          mgm.module_identifier
        )));
        return;
      }
      if mgm.all_dependencies().len() != node.dependencies.len() {
        recovery_error = Some(rspack_error::error!(format!(
          "persistent cache module graph has incomplete dependencies for {}",
          module.identifier()
        )));
        return;
      }
      for (dependency_position, (dep, parent_block, index_in_block)) in
        node.dependencies.into_iter().enumerate()
      {
        let dep = dep.into_owned();
        let dependency_id = *dep.id();
        if let Err(err) = validate_dependency_id_range(dependency_id, max_dependency_id) {
          recovery_error = Some(err);
          return;
        }
        if mgm.all_dependencies()[dependency_position] != dependency_id {
          recovery_error = Some(rspack_error::error!(format!(
            "persistent cache module graph has inconsistent dependency order for {}",
            module.identifier()
          )));
          return;
        }
        if mg.has_dependency(&dependency_id) {
          recovery_error = Some(rspack_error::error!(format!(
            "persistent cache module graph contains duplicate dependency {dependency_id:?}"
          )));
          return;
        }
        mg.set_parents(
          dependency_id,
          DependencyParents {
            block: parent_block.map(|b| b.into_owned()),
            module: module.identifier(),
            index_in_block,
          },
        );
        mg.add_dependency(dep);
      }
      if module.get_blocks().len() != node.blocks.len() {
        recovery_error = Some(rspack_error::error!(format!(
          "persistent cache module graph has incomplete blocks for {}",
          module.identifier()
        )));
        return;
      }
      if mgm.outgoing_connections().len() != node.connections.len() {
        recovery_error = Some(rspack_error::error!(format!(
          "persistent cache module graph has incomplete connections for {}",
          module.identifier()
        )));
        return;
      }
      for con in node.connections {
        let con = con.into_owned();
        let dependency_id = con.dependency_id;
        if !mgm.outgoing_connections().contains(&dependency_id)
          || con.original_module_identifier != Some(module.identifier())
        {
          recovery_error = Some(rspack_error::error!(format!(
            "persistent cache module graph has inconsistent connections for {}",
            module.identifier()
          )));
          return;
        }
        match recover_connection(&mut mg, con) {
          Ok(()) => recovered_connection_ids.push(dependency_id),
          Err(err) => {
            recovery_error = Some(err);
            return;
          }
        }
      }
      for block in node.blocks {
        let block = block.into_owned();
        let block_id = block.identifier();
        if !module.get_blocks().contains(&block_id) {
          recovery_error = Some(rspack_error::error!(format!(
            "persistent cache module graph has an unexpected block {block_id:?} for {}",
            module.identifier()
          )));
          return;
        }
        if mg.block_by_id(&block_id).is_some() {
          recovery_error = Some(rspack_error::error!(format!(
            "persistent cache module graph contains duplicate block {block_id:?}"
          )));
          return;
        }
        mg.add_block(Box::new(block));
      }
      let lazy_info = node.lazy_info.map(OwnedOrRef::into_owned);
      if module.downcast_ref::<TempModule>().is_none()
        && let Err(err) = validate_module_references(&mg, &module, &mgm)
      {
        recovery_error = Some(err);
        return;
      }
      if let Some(lazy_dependencies) = lazy_info.as_ref()
        && let Err(err) = validate_lazy_dependencies(&mg, module.identifier(), lazy_dependencies)
      {
        recovery_error = Some(err);
        return;
      }
      if let Some(lazy_info) = lazy_info {
        module_to_lazy_make.update_module_lazy_dependencies(module.identifier(), Some(lazy_info));
      }
      mg.add_module_graph_module(mgm);
      mg.add_module(module);
    });
  if let Some(err) = recovery_error {
    return Err(err);
  }
  recover_connection_targets(&mut mg, recovered_connection_ids)?;

  // recovery entry
  let mut entry_module: Vec<ModuleIdentifier> = vec![];
  for (_, mgm) in mg.module_graph_modules() {
    if mgm.issuer().identifier().is_none() {
      entry_module.push(mgm.module_identifier);
    };
  }
  let mut entry_dependencies: FxHashSet<DependencyId> = Default::default();
  for mid in entry_module {
    let dep = TempDependency::default();
    let connection = ModuleGraphConnection::new(*dep.id(), None, mid, false);
    entry_dependencies.insert(*dep.id());
    mg.add_dependency(Box::new(dep));
    mg.cache_recovery_connection(connection);
  }

  tracing::debug!("recovery {} module", mg.modules_len());
  Ok((mg, module_to_lazy_make, entry_dependencies))
}

fn validate_dependency_id_range(dependency_id: DependencyId, max_dependency_id: u32) -> Result<()> {
  if dependency_id.as_u32() >= max_dependency_id {
    return Err(rspack_error::error!(format!(
      "persistent cache dependency {dependency_id:?} exceeds the restored meta ID range"
    )));
  }
  Ok(())
}

fn validate_module_references(
  module_graph: &ModuleGraph,
  module: &BoxModule,
  mgm: &ModuleGraphModule,
) -> Result<()> {
  let module_identifier = module.identifier();
  let mut referenced_dependencies = module.get_dependencies().len();
  for (index_in_block, dependency_id) in module.get_dependencies().iter().enumerate() {
    validate_dependency_parent(
      module_graph,
      *dependency_id,
      module_identifier,
      None,
      index_in_block,
    )?;
  }

  for block_id in module.get_blocks() {
    let block = module_graph.block_by_id(block_id).ok_or_else(|| {
      rspack_error::error!(format!(
        "persistent cache module graph references missing block {block_id:?}"
      ))
    })?;
    if block.parent() != &module_identifier {
      return Err(rspack_error::error!(format!(
        "persistent cache block {block_id:?} has an inconsistent parent"
      )));
    }
    referenced_dependencies += block.get_dependencies().len();
    for (index_in_block, dependency_id) in block.get_dependencies().iter().enumerate() {
      validate_dependency_parent(
        module_graph,
        *dependency_id,
        module_identifier,
        Some(block_id),
        index_in_block,
      )?;
    }
  }
  if referenced_dependencies != mgm.all_dependencies().len() {
    return Err(rspack_error::error!(format!(
      "persistent cache module graph has incomplete dependency references for {module_identifier}"
    )));
  }

  Ok(())
}

fn validate_lazy_dependencies(
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
  lazy_dependencies: &LazyDependencies,
) -> Result<()> {
  for dependency_id in lazy_dependencies.all_lazy_dependencies() {
    if !module_graph.has_dependency(&dependency_id)
      || module_graph.get_parent_module(&dependency_id) != Some(&module_identifier)
    {
      return Err(rspack_error::error!(format!(
        "persistent cache module graph has an invalid lazy dependency {dependency_id:?}"
      )));
    }
  }
  Ok(())
}

fn validate_dependency_parent(
  module_graph: &ModuleGraph,
  dependency_id: DependencyId,
  module_identifier: ModuleIdentifier,
  block_id: Option<&AsyncDependenciesBlockIdentifier>,
  index_in_block: usize,
) -> Result<()> {
  if !module_graph.has_dependency(&dependency_id)
    || module_graph.get_parent_module(&dependency_id) != Some(&module_identifier)
    || module_graph.get_parent_block(&dependency_id) != block_id
    || module_graph.get_parent_block_index(&dependency_id) != Some(index_in_block)
  {
    return Err(rspack_error::error!(format!(
      "persistent cache dependency {dependency_id:?} has inconsistent parent metadata"
    )));
  }
  Ok(())
}

fn recover_connection(
  module_graph: &mut ModuleGraph,
  connection: ModuleGraphConnection,
) -> Result<()> {
  let dependency_id = connection.dependency_id;
  if !module_graph.has_dependency(&dependency_id) {
    return Err(rspack_error::error!(format!(
      "persistent cache module graph connection references missing dependency {dependency_id:?}"
    )));
  }
  if module_graph
    .connection_by_dependency_id(&dependency_id)
    .is_some()
  {
    return Err(rspack_error::error!(format!(
      "persistent cache module graph contains duplicate connection {dependency_id:?}"
    )));
  }
  module_graph.cache_recovery_connection(connection);
  Ok(())
}

fn recover_connection_targets(
  module_graph: &mut ModuleGraph,
  connection_ids: Vec<DependencyId>,
) -> Result<()> {
  for dependency_id in connection_ids {
    let connection = module_graph
      .connection_by_dependency_id(&dependency_id)
      .expect("checked connection should exist");
    let target_module = *connection.module_identifier();
    let original_module = connection.original_module_identifier;
    if module_graph
      .module_graph_module_by_identifier(&target_module)
      .is_none()
    {
      return Err(rspack_error::error!(format!(
        "persistent cache module graph connection references missing target module {target_module}"
      )));
    }
    if let Some(original_module) = original_module
      && module_graph
        .module_graph_module_by_identifier(&original_module)
        .is_none()
    {
      return Err(rspack_error::error!(format!(
        "persistent cache module graph connection references missing original module {original_module}"
      )));
    }

    module_graph
      .module_graph_module_by_identifier_mut(&target_module)
      .add_incoming_connection(dependency_id);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use rspack_tasks::{
    set_current_dependency_id, within_compiler_context_for_testing,
    within_compiler_context_for_testing_sync,
  };

  use super::*;
  use crate::{
    cache::persistent::storage::{MemoryStorage, Storage},
    compilation::build_module_graph::LazyUntil,
  };

  #[tokio::test]
  async fn should_return_error_for_corrupted_module_graph() {
    within_compiler_context_for_testing(async {
      let codec = CacheCodec::new(None);
      let mut storage = MemoryStorage::default();
      storage.set(SCOPE, b"module".to_vec(), b"corrupted".to_vec());

      assert!(recovery_module_graph(&storage, &codec).await.is_err());
    })
    .await;
  }

  #[test]
  fn should_return_error_for_connection_with_missing_dependency() {
    within_compiler_context_for_testing_sync(|| {
      let dependency = TempDependency::default();
      let dependency_id = *dependency.id();
      let connection = ModuleGraphConnection::new(
        dependency_id,
        Some(ModuleIdentifier::from("source")),
        ModuleIdentifier::from("target"),
        false,
      );

      assert!(recover_connection(&mut ModuleGraph::default(), connection).is_err());
    });
  }

  #[test]
  fn should_return_error_for_connection_with_missing_target() {
    within_compiler_context_for_testing_sync(|| {
      let dependency = TempDependency::default();
      let dependency_id = *dependency.id();
      let connection = ModuleGraphConnection::new(
        dependency_id,
        None,
        ModuleIdentifier::from("missing-target"),
        false,
      );
      let mut module_graph = ModuleGraph::default();
      module_graph.add_dependency(Box::new(dependency));
      recover_connection(&mut module_graph, connection).unwrap();

      assert!(recover_connection_targets(&mut module_graph, vec![dependency_id]).is_err());
    });
  }

  #[test]
  fn should_reject_dependencies_outside_restored_meta_range() {
    within_compiler_context_for_testing_sync(|| {
      let dependency = TempDependency::default();
      let dependency_id = *dependency.id();
      set_current_dependency_id(dependency_id.as_u32());

      assert!(validate_dependency_id_range(dependency_id, get_current_dependency_id()).is_err());
    });
  }

  #[test]
  fn should_reject_inconsistent_dependency_parent_metadata() {
    within_compiler_context_for_testing_sync(|| {
      let dependency = TempDependency::default();
      let dependency_id = *dependency.id();
      let module_identifier = ModuleIdentifier::from("module");
      let mut module_graph = ModuleGraph::default();
      module_graph.add_dependency(Box::new(dependency));
      module_graph.set_parents(
        dependency_id,
        DependencyParents {
          block: Some(AsyncDependenciesBlockIdentifier::from(
            "missing-block".to_string(),
          )),
          module: module_identifier,
          index_in_block: 0,
        },
      );

      assert!(
        validate_dependency_parent(&module_graph, dependency_id, module_identifier, None, 0)
          .is_err()
      );
    });
  }

  #[test]
  fn should_reject_missing_lazy_dependency() {
    within_compiler_context_for_testing_sync(|| {
      let dependency: BoxDependency = Box::new(TempDependency::default());
      let mut lazy_dependencies = LazyDependencies::default();
      lazy_dependencies.insert(&dependency, LazyUntil::Fallback);

      assert!(
        validate_lazy_dependencies(
          &ModuleGraph::default(),
          ModuleIdentifier::from("module"),
          &lazy_dependencies
        )
        .is_err()
      );
    });
  }
}
