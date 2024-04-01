use std::collections::hash_map::Entry;
use std::{borrow::Cow, hash::BuildHasherDefault};

use dashmap::DashMap;
use itertools::Itertools;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_identifier::IdentifierMap;
use rspack_util::fx_dashmap::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Dependency, ExportMode,
  GetModeCacheKey, ProvidedExports, RuntimeSpec, UsedExports,
};
mod module;
pub use module::*;
mod connection;
pub use connection::*;
mod vec_map;

use crate::{
  BoxDependency, BoxModule, BuildDependency, DependencyCondition, DependencyId, ExportInfo,
  ExportInfoId, ExportsInfo, ExportsInfoId, ModuleIdentifier, ModuleProfile,
};

// TODO Here request can be used Atom
pub type ImportVarMap =
  HashMap<Option<ModuleIdentifier> /* request */, String /* import_var */>;

/// https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L742-L748
#[derive(Debug)]
pub struct DependencyExtraMeta {
  pub ids: Vec<Atom>,
}

#[derive(Debug, Default)]
pub struct DependencyParents {
  pub block: Option<AsyncDependenciesBlockIdentifier>,
  pub module: ModuleIdentifier,
}

#[derive(Debug, Default)]
pub struct ModuleGraphPartial {
  // TODO: removed when new treeshaking is stable
  is_new_treeshaking: bool,

  dependency_id_to_module_identifier: HashMap<DependencyId, Option<ModuleIdentifier>>,

  /// Module identifier to its module
  module_identifier_to_module: IdentifierMap<Option<BoxModule>>,

  /// Module identifier to its module graph module
  module_identifier_to_module_graph_module: IdentifierMap<Option<ModuleGraphModule>>,

  blocks: HashMap<AsyncDependenciesBlockIdentifier, AsyncDependenciesBlock>,

  dependency_id_to_connection_id: HashMap<DependencyId, Option<ConnectionId>>,

  /// Dependencies indexed by `DependencyId`
  /// None means the dependency has been removed
  dependencies: HashMap<DependencyId, BoxDependency>,

  dependency_id_to_parents: HashMap<DependencyId, DependencyParents>,

  /// Dependencies indexed by `ConnectionId`
  connections: HashMap<ConnectionId, Option<ModuleGraphConnection>>,

  exports_info_map: vec_map::VecMap<ExportsInfo>,
  export_info_map: vec_map::VecMap<ExportInfo>,
  connection_to_condition: HashMap<ConnectionId, DependencyCondition>,
  dep_meta_map: HashMap<DependencyId, DependencyExtraMeta>,
  // HACK: temp workaround, remove when rspack has stable cache
  get_mode_cache: FxDashMap<GetModeCacheKey, ExportMode>,
}

impl ModuleGraphPartial {
  // TODO remove and use default() after new_treeshaking stable
  pub fn new(is_new_treeshaking: bool) -> Self {
    Self {
      is_new_treeshaking,
      ..Default::default()
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph<'a> {
  partials: Vec<&'a ModuleGraphPartial>,
  active: Option<&'a mut ModuleGraphPartial>,
}

impl<'a> ModuleGraph<'a> {
  pub fn new(
    partials: Vec<&'a ModuleGraphPartial>,
    active: Option<&'a mut ModuleGraphPartial>,
  ) -> Self {
    Self { partials, active }
  }

  fn loop_partials<T>(&self, f: impl Fn(&ModuleGraphPartial) -> Option<&T>) -> Option<&T> {
    if let Some(active) = &self.active
      && let Some(r) = f(active)
    {
      return Some(r);
    }

    for item in self.partials.iter().rev() {
      if let Some(r) = f(item) {
        return Some(r);
      }
    }
    None
  }

  fn loop_partials_mut<REF, MUT>(
    &mut self,
    f_exist: impl Fn(&ModuleGraphPartial) -> bool,
    f_set: impl Fn(&mut ModuleGraphPartial, REF),
    f: impl Fn(&ModuleGraphPartial) -> Option<REF>,
    f_mut: impl Fn(&mut ModuleGraphPartial) -> Option<&mut MUT>,
  ) -> Option<&mut MUT> {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };

    let active_exist = f_exist(active_partial);
    if !active_exist {
      let mut search_result = None;
      for item in self.partials.iter().rev() {
        if let Some(r) = f(item) {
          search_result = Some(r);
          break;
        }
      }
      if let Some(search_result) = search_result {
        f_set(active_partial, search_result);
      }
    }

    f_mut(active_partial)
  }

  // TODO: removed when new treeshaking is stable
  pub fn is_new_treeshaking(&self) -> bool {
    if let Some(active) = &self.active {
      return active.is_new_treeshaking;
    }
    if let Some(p) = self.partials.last() {
      p.is_new_treeshaking
    } else {
      panic!("can not get any partial module graph")
    }
  }

  /// Return an unordered iterator of modules
  pub fn modules(&self) -> IdentifierMap<&BoxModule> {
    let mut res = IdentifierMap::default();
    for item in self.partials.iter() {
      for (k, v) in &item.module_identifier_to_module {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.module_identifier_to_module {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    res
  }

  pub fn module_graph_modules(&self) -> IdentifierMap<&ModuleGraphModule> {
    let mut res = IdentifierMap::default();
    for item in self.partials.iter() {
      for (k, v) in &item.module_identifier_to_module_graph_module {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.module_identifier_to_module_graph_module {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    res
  }

  pub fn get_incoming_connections_by_origin_module(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> {
    let connections = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have mgm")
      .incoming_connections();

    let mut map: HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> = HashMap::default();
    for connection_id in connections {
      let con = self
        .connection_by_connection_id(connection_id)
        .expect("should have connection");
      match map.entry(con.original_module_identifier) {
        Entry::Occupied(mut occ) => {
          occ.get_mut().push(con.clone());
        }
        Entry::Vacant(vac) => {
          vac.insert(vec![con.clone()]);
        }
      }
    }
    map
  }

  pub fn get_export_mode_cache(&self) -> Option<&FxDashMap<GetModeCacheKey, ExportMode>> {
    if let Some(p) = self.partials.first() {
      return Some(&p.get_mode_cache);
    }

    if let Some(active) = &self.active.as_ref() {
      return Some(&active.get_mode_cache);
    }
    panic!("should have partial")
  }

  pub fn get_export_mode(&self, key: &GetModeCacheKey) -> Option<ExportMode> {
    if let Some(active) = &self
      .active
      .as_ref()
      .and_then(|active| active.get_mode_cache.get(&key))
    {
      dbg!(&active);
      return Some(active.value().clone());
    }

    if let Some(p) = self.partials.last() {
      p.get_mode_cache.get(key).map(|item| item.value().clone())
    } else {
      None
      // panic!("can not get any partial module graph")
    }
  }

  pub fn insert_export_mode(&self, key: GetModeCacheKey, export_mode: ExportMode) {
    if let Some(active_partial) = &self.active {
      active_partial.get_mode_cache.insert(key, export_mode);
      return;
    }
    if let Some(partial) = self.partials.last() {
      partial.get_mode_cache.insert(key, export_mode);
    }
  }

  pub fn unfreeze_export_mode_cache(&mut self) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.get_mode_cache.clear();
  }

  /// Remove a connection and return connection origin module identifier and dependency
  fn revoke_connection(&mut self, connection_id: &ConnectionId) -> Option<BuildDependency> {
    let Some(connection) = self.connection_by_connection_id(connection_id) else {
      return None;
    };
    let module_identifier = *connection.module_identifier();
    let original_module_identifier = connection.original_module_identifier;
    let dependency_id = connection.dependency_id;

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.connections.insert(*connection_id, None);

    // remove dependency
    active_partial
      .dependency_id_to_connection_id
      .insert(dependency_id, None);
    active_partial
      .dependency_id_to_module_identifier
      .insert(dependency_id, None);

    // remove outgoing from original module graph module
    if let Some(original_module_identifier) = &original_module_identifier {
      if let Some(mgm) = self.module_graph_module_by_identifier_mut(original_module_identifier) {
        mgm.remove_outgoing_connection(connection_id);
        // Because of mgm.dependencies is set when original module build success
        // it does not need to remove dependency in mgm.dependencies.
      }
    }
    // remove incoming from module graph module
    if let Some(mgm) = self.module_graph_module_by_identifier_mut(&module_identifier) {
      mgm.remove_incoming_connection(connection_id);
    }

    Some((dependency_id, original_module_identifier))
  }

  pub fn revoke_module(&mut self, module_id: &ModuleIdentifier) -> Vec<BuildDependency> {
    let (outgoing_connections, incoming_connections) = self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        (
          mgm.outgoing_connections().clone(),
          mgm.incoming_connections().clone(),
        )
      })
      .unwrap_or_default();

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .module_identifier_to_module
      .insert(*module_id, None);
    active_partial
      .module_identifier_to_module_graph_module
      .insert(*module_id, None);

    for cid in outgoing_connections {
      self.revoke_connection(&cid);
    }

    incoming_connections
      .iter()
      .filter_map(|cid| self.revoke_connection(cid))
      .collect()
  }

  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    match active_partial
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier)
    {
      Entry::Occupied(mut val) => {
        if val.get().is_none() {
          val.insert(Some(module_graph_module));
        }
      }
      Entry::Vacant(val) => {
        val.insert(Some(module_graph_module));
      }
    }
  }

  /// Make sure both source and target module are exists in module graph
  pub fn clone_module_attributes(
    &mut self,
    source_module: &ModuleIdentifier,
    target_module: &ModuleIdentifier,
  ) {
    let old_mgm = self
      .module_graph_module_by_identifier(source_module)
      .expect("should have mgm");

    // Using this tuple to avoid violating rustc borrow rules
    let assign_tuple = (
      old_mgm.post_order_index,
      old_mgm.pre_order_index,
      old_mgm.depth,
      old_mgm.exports,
      old_mgm.is_async,
    );
    let new_mgm = self
      .module_graph_module_by_identifier_mut(target_module)
      .expect("should have mgm");
    new_mgm.post_order_index = assign_tuple.0;
    new_mgm.pre_order_index = assign_tuple.1;
    new_mgm.depth = assign_tuple.2;
    new_mgm.exports = assign_tuple.3;
    new_mgm.is_async = assign_tuple.4;
  }

  pub fn move_module_connections(
    &mut self,
    old_module: &ModuleIdentifier,
    new_module: &ModuleIdentifier,
    filter_connection: impl Fn(&ModuleGraphConnection, &Box<dyn Dependency>) -> bool,
  ) {
    if old_module == new_module {
      return;
    }

    let outgoing_connections = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm")
      .outgoing_connections()
      .clone();
    // Outgoing connections
    // avoid violating rustc borrow rules
    let mut add_outgoing_connection = vec![];
    let mut delete_outgoing_connection = vec![];
    for connection_id in outgoing_connections.into_iter() {
      let connection = match self.connection_by_connection_id(&connection_id) {
        Some(con) => con,
        // removed
        None => continue,
      };
      let dependency = self
        .dependency_by_id(&connection.dependency_id)
        .expect("should have dependency");
      if filter_connection(connection, dependency) {
        let connection = self
          .connection_by_connection_id_mut(&connection_id)
          .expect("should have connection");
        connection.original_module_identifier = Some(*new_module);
        add_outgoing_connection.push(connection_id);
        delete_outgoing_connection.push(connection_id);
      }
    }

    let new_mgm = self
      .module_graph_module_by_identifier_mut(new_module)
      .expect("should have mgm");
    for c in add_outgoing_connection {
      new_mgm.add_outgoing_connection(c);
    }

    let old_mgm = self
      .module_graph_module_by_identifier_mut(old_module)
      .expect("should have mgm");
    for c in delete_outgoing_connection {
      old_mgm.remove_outgoing_connection(&c);
    }

    let old_mgm = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm");

    // Outgoing connections
    // avoid violating rustc borrow rules
    let mut add_incoming_connection = vec![];
    let mut delete_incoming_connection = vec![];
    for connection_id in old_mgm.incoming_connections().clone().into_iter() {
      let connection = match self.connection_by_connection_id(&connection_id) {
        Some(con) => con,
        None => continue,
      };
      let dependency = self
        .dependency_by_id(&connection.dependency_id)
        .expect("should have dependency");
      // the inactive connection should not be updated
      if filter_connection(connection, dependency) && (connection.conditional || connection.active)
      {
        let connection = self
          .connection_by_connection_id_mut(&connection_id)
          .expect("should have connection");
        let dep_id = connection.dependency_id;
        connection.set_module_identifier(*new_module);

        let Some(active_partial) = &mut self.active else {
          panic!("should have active partial");
        };
        active_partial
          .dependency_id_to_module_identifier
          .insert(dep_id, Some(*new_module));
        add_incoming_connection.push(connection_id);
        delete_incoming_connection.push(connection_id);
      }
    }

    let new_mgm = self
      .module_graph_module_by_identifier_mut(new_module)
      .expect("should have mgm");
    for c in add_incoming_connection {
      new_mgm.add_incoming_connection(c);
    }

    let old_mgm = self
      .module_graph_module_by_identifier_mut(old_module)
      .expect("should have mgm");
    for c in delete_incoming_connection {
      old_mgm.remove_incoming_connection(&c);
    }
  }

  pub fn copy_outgoing_module_connections<F>(
    &mut self,
    old_module: &ModuleIdentifier,
    new_module: &ModuleIdentifier,
    filter_connection: F,
  ) where
    F: Fn(&ModuleGraphConnection, &ModuleGraph) -> bool,
  {
    if old_module == new_module {
      return;
    }

    let old_mgm_connections = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm")
      .get_outgoing_connections_unordered()
      .clone();

    // Outgoing connections
    for connection_id in old_mgm_connections {
      let connection = self
        .connection_by_connection_id(&connection_id)
        .expect("should have connection")
        .clone();
      if filter_connection(&connection, &*self) {
        let new_connection_id = self.clone_module_graph_connection(
          &connection,
          Some(*new_module),
          *connection.module_identifier(),
        );
        let new_mgm = self
          .module_graph_module_by_identifier_mut(new_module)
          .expect("should have mgm");
        new_mgm.add_outgoing_connection(new_connection_id);
      }
    }
  }

  fn clone_module_graph_connection(
    &mut self,
    old_con: &ModuleGraphConnection,
    original_module_identifier: Option<ModuleIdentifier>,
    module_identifier: ModuleIdentifier,
  ) -> ConnectionId {
    let new_connection_id = ConnectionId::new();
    let mut new_connection = old_con.clone();
    new_connection.id = new_connection_id;
    new_connection.original_module_identifier = original_module_identifier;
    new_connection.set_module_identifier(module_identifier);

    let old_condition = self.loop_partials(|p| p.connection_to_condition.get(&old_con.id));
    self.add_connection(new_connection, old_condition.cloned());
    new_connection_id
  }

  pub fn get_depth(&self, module_id: &ModuleIdentifier) -> Option<usize> {
    let mgm = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have module graph module");
    mgm.depth
  }

  pub fn set_depth(&mut self, module_id: ModuleIdentifier, depth: usize) {
    let mgm = self
      .module_graph_module_by_identifier_mut(&module_id)
      .expect("should have module graph module");
    mgm.depth = Some(depth);
  }

  pub fn set_depth_if_lower(&mut self, module_id: &ModuleIdentifier, depth: usize) -> bool {
    let mgm = self
      .module_graph_module_by_identifier_mut(module_id)
      .expect("should have module graph module");
    if let Some(ref mut cur_depth) = mgm.depth {
      if *cur_depth > depth {
        *cur_depth = depth;
        return true;
      }
    } else {
      mgm.depth = Some(depth);
      return true;
    }
    false
  }

  pub fn add_module(&mut self, module: BoxModule) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    match active_partial
      .module_identifier_to_module
      .entry(module.identifier())
    {
      Entry::Occupied(mut val) => {
        if val.get().is_none() {
          val.insert(Some(module));
        }
      }
      Entry::Vacant(val) => {
        val.insert(Some(module));
      }
    }
  }

  pub fn add_block(&mut self, block: AsyncDependenciesBlock) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.blocks.insert(block.identifier(), block);
  }

  pub fn set_parents(&mut self, dependency_id: DependencyId, parents: DependencyParents) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dependency_id_to_parents
      .insert(dependency_id, parents);
  }

  pub fn get_parent_module(&self, dependency_id: &DependencyId) -> Option<&ModuleIdentifier> {
    self.loop_partials(|p| {
      p.dependency_id_to_parents
        .get(dependency_id)
        .map(|p| &p.module)
    })
  }

  pub fn get_parent_block(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<&AsyncDependenciesBlockIdentifier> {
    self.loop_partials(|p| {
      p.dependency_id_to_parents
        .get(dependency_id)
        .and_then(|p| p.block.as_ref())
    })
  }

  pub fn block_by_id(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> Option<&AsyncDependenciesBlock> {
    self.loop_partials(|p| p.blocks.get(block_id))
  }

  pub fn dependencies(&self) -> HashMap<DependencyId, &BoxDependency> {
    let mut res = HashMap::default();
    for item in self.partials.iter() {
      for (k, v) in &item.dependencies {
        res.insert(*k, v);
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.dependencies {
        res.insert(*k, v);
      }
    }

    res
  }

  pub fn add_dependency(&mut self, dependency: BoxDependency) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dependencies
      .insert(*dependency.id(), dependency);
  }

  pub fn dependency_by_id(&self, dependency_id: &DependencyId) -> Option<&BoxDependency> {
    self.loop_partials(|p| p.dependencies.get(dependency_id))
  }

  /// Uniquely identify a module by its dependency
  pub fn module_graph_module_by_dependency_id(
    &self,
    id: &DependencyId,
  ) -> Option<&ModuleGraphModule> {
    self
      .module_identifier_by_dependency_id(id)
      .and_then(|module_identifier| self.module_graph_module_by_identifier(module_identifier))
  }

  pub fn module_identifier_by_dependency_id(&self, id: &DependencyId) -> Option<&ModuleIdentifier> {
    self
      .loop_partials(|p| p.dependency_id_to_module_identifier.get(id))?
      .as_ref()
  }

  pub fn get_module_by_dependency_id(&self, dependency_id: &DependencyId) -> Option<&BoxModule> {
    if let Some(Some(ref module_id)) =
      self.loop_partials(|p| p.dependency_id_to_module_identifier.get(dependency_id))
    {
      self
        .loop_partials(|p| p.module_identifier_to_module.get(module_id))?
        .as_ref()
    } else {
      None
    }
  }

  fn add_connection(
    &mut self,
    connection: ModuleGraphConnection,
    condition: Option<DependencyCondition>,
  ) {
    if self.connection_by_connection_id(&connection.id).is_some() {
      return;
    }

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };

    if let Some(condition) = condition {
      active_partial
        .connection_to_condition
        .insert(connection.id, condition);
    }

    active_partial.dependency_id_to_module_identifier.insert(
      connection.dependency_id,
      Some(*connection.module_identifier()),
    );

    let module_id = *connection.module_identifier();
    let origin_module_id = connection.original_module_identifier;
    let connection_id = connection.id;

    // add to connections list
    active_partial
      .connections
      .insert(connection.id, Some(connection));

    // set to module incoming connection
    {
      let mgm = self
        .module_graph_module_by_identifier_mut(&module_id)
        .unwrap_or_else(|| {
          panic!(
            "Failed to add connection: Module linked to module identifier {module_id} cannot be found"
          )
        });

      mgm.add_incoming_connection(connection_id);
    }

    // set to origin module outgoing connection
    if let Some(identifier) = origin_module_id
      && let Some(original_mgm) = self.module_graph_module_by_identifier_mut(&identifier)
    {
      original_mgm.add_outgoing_connection(connection_id);
    };
  }

  /// Add a connection between two module graph modules, if a connection exists, then it will be reused.
  pub fn set_resolved_module(
    &mut self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let dependency = self
      .dependency_by_id(&dependency_id)
      .expect("should have dependency");
    let is_module_dependency =
      dependency.as_module_dependency().is_some() || dependency.as_context_dependency().is_some();
    let condition = if self.is_new_treeshaking() {
      dependency
        .as_module_dependency()
        .and_then(|dep| dep.get_condition())
    } else {
      None
    };
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dependency_id_to_module_identifier
      .insert(dependency_id, Some(module_identifier));

    if !is_module_dependency {
      return Ok(());
    }

    let active = !matches!(condition, Some(DependencyCondition::False));
    let conditional = condition.is_some();
    let new_connection = ModuleGraphConnection::new(
      original_module_identifier,
      dependency_id,
      module_identifier,
      active,
      conditional,
    );
    active_partial
      .dependency_id_to_connection_id
      .insert(new_connection.dependency_id, Some(new_connection.id));
    self.add_connection(new_connection, condition);

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self
      .loop_partials(|p| p.module_identifier_to_module.get(identifier))?
      .as_ref()
  }

  /// Aggregate function which combine `get_normal_module_by_identifier`, `as_normal_module`, `get_resource_resolved_data`
  pub fn normal_module_source_path_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<Cow<str>> {
    self
      .module_by_identifier(identifier)
      .and_then(|module| module.as_normal_module())
      .map(|module| {
        module
          .resource_resolved_data()
          .resource_path
          .to_string_lossy()
      })
  }

  pub fn connection_id_by_dependency_id(&self, dep_id: &DependencyId) -> Option<&ConnectionId> {
    self
      .loop_partials(|p| p.dependency_id_to_connection_id.get(dep_id))?
      .as_ref()
  }
  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<&ModuleGraphModule> {
    self
      .loop_partials(|p| p.module_identifier_to_module_graph_module.get(identifier))?
      .as_ref()
  }

  /// Uniquely identify a module graph module by its module's identifier and return the exclusive reference
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut ModuleGraphModule> {
    self
      .loop_partials_mut(
        |p| {
          p.module_identifier_to_module_graph_module
            .contains_key(identifier)
        },
        |p, search_result| {
          p.module_identifier_to_module_graph_module
            .insert(*identifier, search_result);
        },
        |p| {
          p.module_identifier_to_module_graph_module
            .get(identifier)
            .cloned()
        },
        |p| {
          p.module_identifier_to_module_graph_module
            .get_mut(identifier)
        },
      )?
      .as_mut()
  }

  /// refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L582-L585
  pub fn get_export_info(
    &mut self,
    module_id: ModuleIdentifier,
    export_name: &Atom,
  ) -> ExportInfoId {
    let exports_info_id = self.get_exports_info(&module_id).id;
    exports_info_id.get_export_info(export_name, self)
  }
  /// Uniquely identify a connection by a given dependency
  pub fn connection_by_dependency(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<&ModuleGraphConnection> {
    if let Some(connection_id) = self.connection_id_by_dependency_id(dependency_id) {
      self
        .loop_partials(|p| p.connections.get(connection_id))?
        .as_ref()
    } else {
      None
    }
  }

  pub(crate) fn get_ordered_connections(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<Vec<&ConnectionId>> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|m| {
        m.__deprecated_all_dependencies
          .iter()
          .filter_map(|dep_id| self.connection_id_by_dependency_id(dep_id))
          .collect()
      })
  }

  /// # Deprecated!!!
  /// # Don't use this anymore!!!
  /// A module is a DependenciesBlock, which means it has some Dependencies and some AsyncDependenciesBlocks
  /// a static import is a Dependency, but a dynamic import is a AsyncDependenciesBlock
  /// AsyncDependenciesBlock means it is a code-splitting point, and will create a ChunkGroup in code-splitting
  /// and AsyncDependenciesBlock also is DependenciesBlock, so it can has some Dependencies and some AsyncDependenciesBlocks
  /// so if you want get a module's dependencies and its blocks' dependencies (all dependencies)
  /// just use module.get_dependencies() and module.get_blocks().map(|b| b.get_dependencyes())
  /// you don't need this one
  pub(crate) fn get_module_all_dependencies(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&[DependencyId]> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|m| &*m.__deprecated_all_dependencies)
  }

  /// # Deprecated!!!
  /// # Don't use this anymore!!!
  /// A module is a DependenciesBlock, which means it has some Dependencies and some AsyncDependenciesBlocks
  /// a static import is a Dependency, but a dynamic import is a AsyncDependenciesBlock
  /// AsyncDependenciesBlock means it is a code-splitting point, and will create a ChunkGroup in code-splitting
  /// and AsyncDependenciesBlock also is DependenciesBlock, so it can has some Dependencies and some AsyncDependenciesBlocks
  /// so if you want get a module's dependencies and its blocks' dependencies (all dependencies)
  /// just use module.get_dependencies() and module.get_blocks().map(|b| b.get_dependencyes())
  /// you don't need this one
  pub(crate) fn get_module_all_depended_modules(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<Vec<&ModuleIdentifier>> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|m| {
        m.__deprecated_all_dependencies
          .iter()
          .filter_map(|id| self.module_identifier_by_dependency_id(id))
          .collect()
      })
  }

  pub(crate) fn get_module_dependencies_modules_and_blocks(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> (Vec<ModuleIdentifier>, &[AsyncDependenciesBlockIdentifier]) {
    let Some(m) = self.module_by_identifier(module_identifier) else {
      unreachable!("cannot find the module correspanding to {module_identifier}");
    };
    let mut deps = m
      .get_dependencies()
      .iter()
      .filter_map(|id| self.dependency_by_id(id))
      .filter(|dep| dep.as_module_dependency().is_some())
      .collect::<Vec<_>>();

    // sort by span, so user change import order can recalculate chunk graph
    deps.sort_by(|a, b| {
      if let (Some(span_a), Some(span_b)) = (a.span(), b.span()) {
        span_a.cmp(&span_b)
      } else if let (Some(a), Some(b)) = (a.source_order(), b.source_order()) {
        a.cmp(&b)
      } else {
        a.id().cmp(b.id())
      }
    });

    let modules = deps
      .into_iter()
      .filter_map(|dep| self.module_identifier_by_dependency_id(dep.id()))
      .dedup_by(|a, b| a.as_str() == b.as_str())
      .copied()
      .collect();
    let blocks = m.get_blocks();
    (modules, blocks)
  }

  pub fn parent_module_by_dependency_id(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<ModuleIdentifier> {
    self
      .connection_by_dependency(dependency_id)
      .and_then(|c| c.original_module_identifier)
  }

  pub fn connection_by_connection_id(
    &self,
    connection_id: &ConnectionId,
  ) -> Option<&ModuleGraphConnection> {
    self
      .loop_partials(|p| p.connections.get(connection_id))?
      .as_ref()
  }

  pub fn connection_by_connection_id_mut(
    &mut self,
    connection_id: &ConnectionId,
  ) -> Option<&mut ModuleGraphConnection> {
    self
      .loop_partials_mut(
        |p| p.connections.contains_key(connection_id),
        |p, search_result| {
          p.connections.insert(*connection_id, search_result);
        },
        |p| p.connections.get(connection_id).cloned(),
        |p| p.connections.get_mut(connection_id),
      )?
      .as_mut()
  }

  pub fn get_pre_order_index(&self, module_id: &ModuleIdentifier) -> Option<u32> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.pre_order_index)
  }

  pub fn get_issuer(&self, module_id: &ModuleIdentifier) -> Option<&BoxModule> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.get_issuer().get_module(self))
  }

  pub fn is_async(&self, module_id: &ModuleIdentifier) -> Option<bool> {
    self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| mgm.is_async)
  }

  pub fn set_async(&mut self, module_id: &ModuleIdentifier) {
    if let Some(mgm) = self.module_graph_module_by_identifier_mut(module_id) {
      mgm.is_async = true
    }
  }

  pub fn get_outgoing_connections(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        mgm
          .outgoing_connections()
          .iter()
          .filter_map(|id| self.connection_by_connection_id(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_incoming_connections(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        mgm
          .incoming_connections()
          .iter()
          .filter_map(|id| self.connection_by_connection_id(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_profile(&self, module_id: &ModuleIdentifier) -> Option<&ModuleProfile> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.get_profile())
  }

  pub fn get_module_hash(&self, module_id: &ModuleIdentifier) -> Option<&RspackHashDigest> {
    self
      .module_by_identifier(module_id)
      .and_then(|mgm| mgm.build_info().as_ref().and_then(|i| i.hash.as_ref()))
  }

  /// We can't insert all sort of things into one hashmap like javascript, so we create different
  /// hashmap to store different kinds of meta.
  pub fn get_dep_meta_if_existing(&self, id: &DependencyId) -> Option<&DependencyExtraMeta> {
    self.loop_partials(|p| p.dep_meta_map.get(id))
  }

  pub fn set_dep_meta(&mut self, dep_id: DependencyId, ids: Vec<Atom>) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dep_meta_map
      .insert(dep_id, DependencyExtraMeta { ids });
  }

  pub fn update_module(&mut self, dep_id: &DependencyId, module_id: &ModuleIdentifier) {
    let connection_id = *self
      .connection_id_by_dependency_id(dep_id)
      .expect("should have connection id");
    let connection = self
      .connection_by_connection_id_mut(&connection_id)
      .expect("should have connection");
    if connection.module_identifier() == module_id {
      return;
    }

    // clone connection
    let mut new_connection = connection.clone();
    new_connection.id = ConnectionId::new();
    let new_connection_id = new_connection.id;

    let old_connection_id = connection.id;
    let old_connection_dependency_id = connection.dependency_id;

    // modify old connection
    connection.set_active(false);

    // copy condition
    let condition = self
      .loop_partials(|p| p.connection_to_condition.get(&old_connection_id))
      .cloned();

    new_connection.set_module_identifier(*module_id);
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dependency_id_to_module_identifier
      .insert(new_connection.dependency_id, Some(*module_id));
    // add new connection
    active_partial
      .connections
      .insert(new_connection_id, Some(new_connection.clone()));

    active_partial
      .dependency_id_to_connection_id
      .insert(old_connection_dependency_id, Some(new_connection_id));

    if let Some(condition) = condition {
      active_partial
        .connection_to_condition
        .insert(new_connection_id, condition);
    }

    // add new connection to original_module outgoing connections
    if let Some(ref original_module_identifier) = new_connection.original_module_identifier {
      if let Some(mgm) = self.module_graph_module_by_identifier_mut(original_module_identifier) {
        mgm.add_outgoing_connection(new_connection_id);
        mgm.remove_outgoing_connection(&connection_id);
      }
    }
    // add new connection to module incoming connections
    if let Some(mgm) =
      self.module_graph_module_by_identifier_mut(new_connection.module_identifier())
    {
      mgm.add_incoming_connection(new_connection_id);
      mgm.remove_incoming_connection(&connection_id);
    }
  }

  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> &ExportsInfo {
    let mgm = self
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    self
      .loop_partials(|p| p.exports_info_map.try_get(*mgm.exports as usize))
      .expect("should have exports info")
  }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfoId) -> &ExportsInfo {
    self
      .try_get_exports_info_by_id(id)
      .expect("should have exports info")
  }

  pub fn try_get_exports_info_by_id(&self, id: &ExportsInfoId) -> Option<&ExportsInfo> {
    self.loop_partials(|p| p.exports_info_map.try_get((**id) as usize))
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfoId) -> &mut ExportsInfo {
    let id = (**id) as usize;
    self
      .loop_partials_mut(
        |p| p.exports_info_map.try_get(id).is_some(),
        |p, search_result| {
          p.exports_info_map.insert(id, search_result);
        },
        |p| p.exports_info_map.try_get(id).cloned(),
        |p| p.exports_info_map.try_get_mut(id),
      )
      .expect("should have exports info")
  }

  pub fn set_exports_info(&mut self, id: ExportsInfoId, info: ExportsInfo) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.exports_info_map.insert(*id as usize, info);
  }

  pub fn try_get_export_info_by_id(&self, id: &ExportInfoId) -> Option<&ExportInfo> {
    self.loop_partials(|p| p.export_info_map.try_get((**id) as usize))
  }

  pub fn get_export_info_by_id(&self, id: &ExportInfoId) -> &ExportInfo {
    self
      .try_get_export_info_by_id(id)
      .expect("should have export info")
  }

  pub fn get_export_info_mut_by_id(&mut self, id: &ExportInfoId) -> &mut ExportInfo {
    let id = **id as usize;
    self
      .loop_partials_mut(
        |p| p.export_info_map.try_get(id).is_some(),
        |p, search_result| {
          p.export_info_map.insert(id, search_result);
        },
        |p| p.export_info_map.try_get(id).cloned(),
        |p| p.export_info_map.try_get_mut(id),
      )
      .expect("should have export info")
  }

  pub fn set_export_info(&mut self, id: ExportInfoId, info: ExportInfo) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.export_info_map.insert(*id as usize, info);
  }

  pub fn get_provided_exports(&self, module_id: ModuleIdentifier) -> ProvidedExports {
    let mgm = self
      .module_graph_module_by_identifier(&module_id)
      .expect("should have module graph module");
    mgm
      .exports
      .get_exports_info(self)
      .get_provided_exports(self)
  }

  pub fn get_used_exports(
    &self,
    id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    let mgm = self
      .module_graph_module_by_identifier(id)
      .expect("should have module graph module");
    mgm
      .exports
      .get_exports_info(self)
      .get_used_exports(self, runtime)
  }

  pub fn get_optimization_bailout_mut(&mut self, id: &ModuleIdentifier) -> &mut Vec<String> {
    let mgm = self
      .module_graph_module_by_identifier_mut(id)
      .expect("should have module graph module");
    mgm.optimization_bailout_mut()
  }

  pub fn get_read_only_export_info(
    &self,
    id: &ModuleIdentifier,
    name: Atom,
  ) -> Option<&ExportInfo> {
    self
      .module_graph_module_by_identifier(id)
      .map(|mgm| mgm.exports.get_read_only_export_info(&name, self))
  }

  pub fn get_condition_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
  ) -> ConnectionState {
    let condition = self
      .loop_partials(|p| p.connection_to_condition.get(&connection.id))
      .expect("should have condition");
    match condition {
      DependencyCondition::False => ConnectionState::Bool(false),
      DependencyCondition::Fn(f) => f(connection, runtime, self),
    }
  }
}
