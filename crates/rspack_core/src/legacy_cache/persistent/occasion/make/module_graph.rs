use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::{iter::Either, prelude::*};
use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::{
  super::BoundEntry,
  alternatives::{TempDependency, TempModule},
};
use crate::{
  AsyncDependenciesBlockIdentifier, AsyncDependenciesBlockRef, BoxDependency, CompilationLogger,
  DependenciesBlock, Dependency, DependencyId, DependencyParents, DependencyRef,
  FactorizationArtifact, FactorizeInfo, Logger, ModuleGraph, ModuleGraphConnection,
  ModuleGraphModule, ModuleIdentifier, ModuleRef, RayonConsumer,
  cache::CacheCodec,
  compilation::build_module_graph::{LazyDependencies, ModuleToLazyMake},
  legacy_cache::persistent::storage::Storage,
};

pub const SCOPE: &str = "occasion_make_module_graph";

type CachedDependency<'a> = (
  OwnedOrRef<'a, DependencyRef>,
  Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
  Option<OwnedOrRef<'a, FactorizeInfo>>,
);

/// The value struct of current storage scope
#[cacheable]
struct Node<'a> {
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  pub module: OwnedOrRef<'a, ModuleRef>,
  pub dependencies: Vec<CachedDependency<'a>>,
  pub connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlockRef>>,
  pub lazy_info: Option<OwnedOrRef<'a, LazyDependencies>>,
}

pub struct RecoveredModuleGraph {
  pub mg: ModuleGraph,
  pub factorization_artifact: FactorizationArtifact,
  pub module_to_lazy_make: ModuleToLazyMake,
  pub entry_dependencies: FxHashSet<DependencyId>,
  /// Storage keys of nodes bound to another root.
  pub dropped: Vec<Vec<u8>>,
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::save", skip_all)]
pub fn save_module_graph(
  mg: &ModuleGraph,
  factorization_artifact: &FactorizationArtifact,
  module_to_lazy_make: &ModuleToLazyMake,
  removed_modules: &IdentifierSet,
  need_update_modules: &IdentifierSet,
  storage: &mut dyn Storage,
  codec: &CacheCodec,
) {
  for identifier in removed_modules {
    storage.remove(
      SCOPE,
      &codec.encode(identifier).expect("should encode identifier"),
    );
  }

  let node_codec = if codec.root_hash().is_some() && !need_update_modules.is_empty() {
    let ranges = mg
      .modules()
      .filter_map(|(identifier, module)| {
        let range = module.as_context_module()?.identifier_regex_range()?;
        Some((identifier.to_string(), range))
      })
      .chain(mg.dependencies().filter_map(|(_, dependency)| {
        let dependency = dependency.as_context_dependency()?;
        let range = dependency.resource_identifier_regex_range()?;
        Some((
          crate::ContextDependency::resource_identifier(dependency).to_string(),
          range,
        ))
      }))
      .collect();
    Some(codec.with_context_identifier_regex_ranges(ranges))
  } else {
    None
  };
  let node_codec = node_codec.as_ref().unwrap_or(codec);

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
        .map(|block_id| {
          mg.block_ref_by_id(block_id)
            .expect("should have block")
            .into()
        })
        .collect::<Vec<_>>();
      let dependencies = mgm
        .all_dependencies()
        .par_iter()
        .map(|dep_id| {
          (
            mg.dependency_ref_by_id(dep_id).into(),
            mg.get_parent_block(dep_id).map(Into::into),
            factorization_artifact.get_by_owner(dep_id).map(Into::into),
          )
        })
        .collect::<Vec<_>>();
      let connections = mgm
        .outgoing_connections()
        .par_iter()
        .map(|connection_id| {
          mg.connection_by_id(connection_id)
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
      let key = codec.encode(identifier).expect("should encode identifier");
      let root_sensitive = module.build_info().root_sensitive;
      match BoundEntry::encode(node_codec, &node, root_sensitive) {
        Ok(bytes) => (key, bytes),
        Err(err) if err.to_string().contains("unsupported field") => {
          tracing::warn!("to bytes failed {:?}", err);
          // try use alternatives
          node.module = TempModule::transform_from(node.module, Vec::new());
          node.dependencies = node
            .dependencies
            .into_iter()
            .map(|(dep, _, factorize_info)| {
              (TempDependency::transform_from(dep), None, factorize_info)
            })
            .collect();
          node.blocks = vec![];
          if let Ok(bytes) = BoundEntry::encode(node_codec, &node, root_sensitive) {
            (key, bytes)
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
  logger: &CompilationLogger,
) -> Result<RecoveredModuleGraph> {
  let mut mg = ModuleGraph::default();
  let mut factorization_artifact = FactorizationArtifact::default();
  let mut module_to_lazy_make = ModuleToLazyMake::default();
  let (nodes, dropped): (Vec<_>, Vec<_>) = storage
    .load(SCOPE)
    .await?
    .into_par_iter()
    .filter_map(
      |(key, value)| match BoundEntry::decode::<Node>(codec, &value) {
        Ok(Some(node)) => Some(Either::Left(node)),
        Ok(None) => Some(Either::Right(key)),
        Err(err) => {
          logger.warn(format!("make persistent cache decode failed: {err}"));
          None
        }
      },
    )
    .partition_map(|node| node);
  let recovered = nodes
    .iter()
    .map(|node| node.module.as_ref().identifier())
    .collect::<IdentifierSet>();
  let mut connections = vec![];
  for node in nodes {
    let mut mgm = node.mgm.into_owned();
    let mut module = node.module.into_owned();
    for (index_in_block, (dep, parent_block, factorize_info)) in
      node.dependencies.into_iter().enumerate()
    {
      let dep = dep.into_owned();
      mg.set_parents(
        *dep.id(),
        DependencyParents {
          block: parent_block.map(|b| b.into_owned()),
          module: module.identifier(),
          index_in_block,
        },
      );
      mg.add_dependency_ref(dep);
      if let Some(factorize_info) = factorize_info {
        factorization_artifact.insert(factorize_info.into_owned());
      }
    }
    let mut missing_target = false;
    for con in node.connections {
      let con = con.into_owned();
      if recovered.contains(con.module_identifier()) {
        connections.push(con);
      } else {
        mgm.remove_outgoing_connection(&con.id);
        missing_target = true;
      }
    }
    if missing_target {
      // Rebuild the owner through the normal cutout path to refactorize its missing targets.
      let blocks = module.get_block_refs().to_vec();
      module = TempModule::transform_from(module.into(), blocks).into_owned();
    }
    if mgm
      .issuer()
      .identifier()
      .is_some_and(|issuer| !recovered.contains(issuer))
    {
      mgm.set_issuer(crate::ModuleIssuer::Unset);
    }
    let mut blocks = node
      .blocks
      .into_iter()
      .map(OwnedOrRef::into_owned)
      .collect::<Vec<_>>();
    while let Some(block) = blocks.pop() {
      blocks.extend(block.get_block_refs().iter().cloned());
      mg.add_block(block);
    }
    if let Some(lazy_info) = node.lazy_info {
      module_to_lazy_make
        .update_module_lazy_dependencies(module.identifier(), Some(lazy_info.into_owned()));
    }
    mg.add_module_graph_module(mgm);
    mg.add_module(module);
  }
  // recovery incoming connections
  for connection in connections {
    let mgm = mg.module_graph_module_by_identifier_mut(connection.module_identifier());
    mgm.add_incoming_connection(connection.id);
    mg.cache_recovery_connection(connection);
  }
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
    let connection =
      ModuleGraphConnection::new(mg.next_connection_id(), *dep.id(), None, mid, false);
    entry_dependencies.insert(*dep.id());
    mg.add_dependency(BoxDependency::new(dep));
    mg.cache_recovery_connection(connection);
  }

  tracing::debug!("recovery {} module", mg.modules_len());
  Ok(RecoveredModuleGraph {
    mg,
    factorization_artifact,
    module_to_lazy_make,
    entry_dependencies,
    dropped,
  })
}
