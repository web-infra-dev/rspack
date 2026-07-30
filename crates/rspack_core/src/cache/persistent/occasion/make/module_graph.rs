use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::{Result, error};
use rustc_hash::FxHashSet;

use super::alternatives::{TempDependency, TempModule};
use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, Dependency,
  DependencyId, DependencyParents, Module, ModuleGraph, ModuleGraphConnection, ModuleGraphModule,
  ModuleIdentifier, ModuleType, NormalModule, ParseOption, RayonConsumer,
  cache::persistent::{codec::CacheCodec, storage::Storage},
  compilation::build_module_graph::{LazyDependencies, ModuleToLazyMake},
  json_archive::{JsonArchivePolicy, restore_json_data_from_source},
};

pub const SCOPE: &str = "occasion_make_module_graph";

/// The value struct of current storage scope
#[cacheable]
struct Node<'a> {
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  pub json_archive_policy: JsonArchivePolicy,
  pub module: OwnedOrRef<'a, BoxModule>,
  pub dependencies: Vec<(
    OwnedOrRef<'a, BoxDependency>,
    Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
  )>,
  pub connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
  pub lazy_info: Option<OwnedOrRef<'a, LazyDependencies>>,
}

fn json_archive_policy(module: &BoxModule) -> JsonArchivePolicy {
  let Some(module) = module.as_ref().downcast_ref::<NormalModule>() else {
    return JsonArchivePolicy::Preserve;
  };
  if module.module_type() != &ModuleType::Json
    || !module
      .parser_and_generator()
      .can_restore_json_data_from_source()
    || module.source().is_none()
    || module.build_info().json_data.is_none()
  {
    return JsonArchivePolicy::Preserve;
  }

  match module
    .get_parser_options()
    .and_then(|options| options.get_json())
  {
    None => JsonArchivePolicy::DeriveFromModuleSource,
    Some(options) if matches!(&options.parse, ParseOption::None) => {
      JsonArchivePolicy::DeriveFromModuleSource
    }
    Some(_) => JsonArchivePolicy::Preserve,
  }
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
        json_archive_policy: json_archive_policy(module),
        module: module.into(),
        dependencies,
        connections,
        blocks,
        lazy_info,
      };
      match codec.encode_with_json_policy(&node, node.json_archive_policy) {
        Ok(bytes) => (identifier.as_bytes().to_vec(), bytes),
        Err(err) if err.to_string().contains("unsupported field") => {
          tracing::warn!("to bytes failed {:?}", err);
          // try use alternatives
          node.module = TempModule::transform_from(node.module);
          node.json_archive_policy = JsonArchivePolicy::Preserve;
          node.dependencies = node
            .dependencies
            .into_iter()
            .map(|(dep, _)| (TempDependency::transform_from(dep), None))
            .collect();
          node.blocks = vec![];
          if let Ok(bytes) = codec.encode_with_json_policy(&node, node.json_archive_policy) {
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
  let mut need_check_dep = vec![];
  let mut mg = ModuleGraph::default();
  let mut module_to_lazy_make = ModuleToLazyMake::default();
  let mut recovery_error = None;
  storage
    .load(SCOPE)
    .await?
    .into_par_iter()
    .map(|(_, value)| {
      let mut node = codec.decode::<Node>(&value)?;
      if node.json_archive_policy == JsonArchivePolicy::DeriveFromModuleSource {
        let OwnedOrRef::Owned(module) = &mut node.module else {
          return Err(error!(
            "cannot recover source-derived JSON from a borrowed module"
          ));
        };
        let module = module
          .as_mut()
          .downcast_mut::<NormalModule>()
          .ok_or_else(|| error!("cannot recover source-derived JSON for a non-normal module"))?;
        if module.module_type() != &ModuleType::Json {
          return Err(error!(
            "cannot recover source-derived JSON for a non-JSON module"
          ));
        }
        if !module
          .parser_and_generator()
          .can_restore_json_data_from_source()
        {
          return Err(error!(
            "cannot recover source-derived JSON for a custom parser"
          ));
        }
        if module.build_info().json_data.is_some() {
          return Err(error!(
            "source-derived JSON unexpectedly retained canonical data"
          ));
        }
        if module
          .get_parser_options()
          .and_then(|options| options.get_json())
          .is_some_and(|options| !matches!(&options.parse, ParseOption::None))
        {
          return Err(error!(
            "cannot recover source-derived JSON with a custom parse callback"
          ));
        }
        let source = module
          .source()
          .ok_or_else(|| error!("cannot recover JSON module without its source"))?
          .source()
          .into_string_lossy();
        let data = restore_json_data_from_source(&source)?;
        module.build_info_mut().json_data = Some(data);
      }
      Ok(node)
    })
    .with_max_len(1)
    .consume(|result| {
      let node = match result {
        Ok(node) if recovery_error.is_none() => node,
        Ok(_) => return,
        Err(error) => {
          recovery_error.get_or_insert(error);
          return;
        }
      };
      let mgm = node.mgm.into_owned();
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
      mg.add_module_graph_module(mgm);
      mg.add_module(module);
    });
  if let Some(error) = recovery_error {
    return Err(error);
  }
  // recovery incoming connections
  for (dep_id, module_identifier) in need_check_dep {
    let mgm = mg.module_graph_module_by_identifier_mut(&module_identifier);
    mgm.add_incoming_connection(dep_id);
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
    let connection = ModuleGraphConnection::new(*dep.id(), None, mid, false);
    entry_dependencies.insert(*dep.id());
    mg.add_dependency(Box::new(dep));
    mg.cache_recovery_connection(connection);
  }

  tracing::debug!("recovery {} module", mg.modules_len());
  Ok((mg, module_to_lazy_make, entry_dependencies))
}
