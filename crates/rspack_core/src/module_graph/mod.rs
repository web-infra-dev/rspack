use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::path::PathBuf;

use dashmap::DashMap;
use itertools::Itertools;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_identifier::{Identifiable, IdentifierMap};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, ProvidedExports, RuntimeSpec,
  UsedExports,
};
mod module;
pub use module::*;
mod connection;
pub use connection::*;
mod vec_map;

use crate::{
  BoxDependency, BoxModule, BuildDependency, BuildInfo, BuildMeta, DependencyCondition,
  DependencyId, ExportInfo, ExportInfoId, ExportsInfo, ExportsInfoId, ModuleIdentifier,
  ModuleProfile,
};

// TODO Here request can be used Atom
pub type ImportVarMap = HashMap<Option<String> /* request */, String /* import_var */>;

#[derive(Debug, Default)]
pub struct DependencyParents {
  pub block: Option<AsyncDependenciesBlockIdentifier>,
  pub module: ModuleIdentifier,
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  // TODO: removed when new treeshaking is stable
  pub is_new_treeshaking: bool,

  pub dependency_id_to_module_identifier: HashMap<DependencyId, ModuleIdentifier>,

  /// Module identifier to its module
  pub module_identifier_to_module: IdentifierMap<BoxModule>,

  /// Module identifier to its module graph module
  pub module_identifier_to_module_graph_module: IdentifierMap<ModuleGraphModule>,

  blocks: HashMap<AsyncDependenciesBlockIdentifier, AsyncDependenciesBlock>,

  pub dependency_id_to_connection_id: HashMap<DependencyId, ConnectionId>,
  connection_id_to_dependency_id: HashMap<ConnectionId, DependencyId>,

  /// Dependencies indexed by `DependencyId`
  /// None means the dependency has been removed
  dependencies: HashMap<DependencyId, BoxDependency>,

  dependency_id_to_parents: HashMap<DependencyId, DependencyParents>,

  /// Dependencies indexed by `ConnectionId`
  /// None means the connection has been removed
  pub connections: Vec<Option<ModuleGraphConnection>>,

  /// Module graph connections table index for `ConnectionId`
  connections_map: HashMap<ModuleGraphConnection, ConnectionId>,

  pub import_var_map: DashMap<ModuleIdentifier, ImportVarMap>,
  pub exports_info_hash: DashMap<ExportsInfoId, u64>,
  pub exports_info_map: vec_map::VecMap<ExportsInfo>,
  pub export_info_map: vec_map::VecMap<ExportInfo>,
  connection_to_condition: HashMap<ModuleGraphConnection, DependencyCondition>,
  pub dep_meta_map: HashMap<DependencyId, DependencyExtraMeta>,
}

/// https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L742-L748
#[derive(Debug)]
pub struct DependencyExtraMeta {
  pub ids: Vec<Atom>,
}

impl ModuleGraph {
  // TODO: removed when new treeshaking is stable
  pub fn with_treeshaking(mut self, new_treeshaking: bool) -> Self {
    self.is_new_treeshaking = new_treeshaking;
    self
  }

  /// Return an unordered iterator of modules
  pub fn modules(&self) -> &IdentifierMap<BoxModule> {
    &self.module_identifier_to_module
  }

  pub fn module_graph_modules(&self) -> &IdentifierMap<ModuleGraphModule> {
    &self.module_identifier_to_module_graph_module
  }

  pub fn get_incoming_connections_by_origin_module(
    &self,
    module: &ModuleIdentifier,
  ) -> HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> {
    let connections = &self
      .module_graph_module_by_identifier(module)
      .expect("should have mgm")
      .incoming_connections();
    get_connections_by_origin_module(connections.iter(), self)
  }

  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    if let Entry::Vacant(val) = self
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier)
    {
      val.insert(module_graph_module);
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

  pub fn move_module_connections<F>(
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

    let old_mgm = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm");

    // Outgoing connections
    // avoid violating rustc borrow rules
    let mut add_outgoing_connection = vec![];
    let mut delete_outgoing_connection = vec![];
    for connection_id in old_mgm.outgoing_connections().clone().into_iter() {
      let connection = self
        .connection_by_connection_id(&connection_id)
        .cloned()
        .expect("should have connection");
      if filter_connection(&connection, &*self) {
        let connection = self
          .connection_by_connection_id_mut(&connection_id)
          .expect("should have connection");
        connection.original_module_identifier = Some(*new_module);
        // dbg!(&self.dependency_id_to_parents.get(&connection.dependency_id));
        // let new_connection_id = self.clone_module_graph_connection(
        //   &connection,
        //   Some(*new_module),
        //   connection.module_identifier,
        // );
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
      old_mgm.remove_outgoing_connection(c);
    }

    let old_mgm = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm");

    // Outgoing connections
    // avoid violating rustc borrow rules
    let mut add_incoming_connection = vec![];
    let mut delete_incoming_connection = vec![];
    for connection_id in old_mgm.incoming_connections().clone().into_iter() {
      let connection = self
        .connection_by_connection_id(&connection_id)
        .cloned()
        .expect("should have connection");
      if filter_connection(&connection, &*self) {
        // let new_connection_id = self.clone_module_graph_connection(
        //   &connection,
        //   connection.original_module_identifier,
        //   *new_module,
        // );

        let connection = self
          .connection_by_connection_id_mut(&connection_id)
          .expect("should have connection");
        dbg!(&connection.module_identifier, &new_module,);
        connection.module_identifier = *new_module;
        // TODO: recover
        let dep_id = connection.dependency_id;
        // dbg!(&dep_id.get_dependency(self).get_ids(self));
        let dep = dep_id.get_dependency(self);
        // dep.dependency_debug_name() != "HarmonyExportImportedSpecifierDependency"
        // if dep.dependency_debug_name() != "HarmonyImportSpecifierDependency" {
        self
          .dependency_id_to_module_identifier
          .insert(dep_id, *new_module);
        // }
        //   .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
        //   .is_some();
        // let is_valid_import_specifier_dep = dep
        //   .downcast_ref::<HarmonyImportSpecifierDependency>()
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
      old_mgm.remove_incoming_connection(c);
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

    let old_mgm = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm");
    let old_connections = old_mgm
      .get_outgoing_connections_unordered(self)
      .map(|cons| cons.copied().collect::<Vec<_>>());

    // Outgoing connections
    let mut pairs = vec![];
    if let Ok(old_connections) = old_connections {
      for connection in old_connections.into_iter() {
        if filter_connection(&connection, &*self) {
          let new_connection_id = self.clone_module_graph_connection(
            &connection,
            Some(*new_module),
            connection.module_identifier,
          );
          let new_mgm = self
            .module_graph_module_by_identifier_mut(new_module)
            .expect("should have mgm");
          new_mgm.add_outgoing_connection(new_connection_id);
          pairs.push((connection.module_identifier, new_connection_id));
        }
      }
      for (k, v) in pairs {
        let old_mgm = self
          .module_graph_module_by_identifier_mut(&k)
          .expect("should have mgm");
        old_mgm.add_incoming_connection(v);
      }
    }
  }

  pub fn clone_module_graph_connection(
    &mut self,
    old_con: &ModuleGraphConnection,
    original_module_identifier: Option<ModuleIdentifier>,
    module_identifier: ModuleIdentifier,
  ) -> ConnectionId {
    // let old_con_id = self.connection_id_by_dependency_id(&old_con.dependency_id);
    let mut new_connection = *old_con;
    new_connection.original_module_identifier = original_module_identifier;
    new_connection.module_identifier = module_identifier;

    let new_connection_id = {
      let new_connection_id = ConnectionId::from(self.connections.len());
      self.connections.push(Some(new_connection));
      self
        .connections_map
        .insert(new_connection, new_connection_id);
      new_connection_id
    };

    // clone condition
    if let Some(condition) = self.connection_to_condition.get(old_con) {
      self
        .connection_to_condition
        .insert(new_connection, condition.clone());
    }

    self
      .dependency_id_to_module_identifier
      .insert(new_connection.dependency_id, module_identifier);

    self
      .connection_id_to_dependency_id
      .insert(new_connection_id, new_connection.dependency_id);

    {
      let mgm = self
        .module_graph_module_by_identifier_mut(&module_identifier)
        .unwrap_or_else(|| {
          panic!(
            "Failed to set resolved module: Module linked to module identifier {module_identifier} cannot be found"
          )
        });

      mgm.add_incoming_connection(new_connection_id);
    }

    if let Some(identifier) = original_module_identifier
      && let Some(original_mgm) = self.module_graph_module_by_identifier_mut(&identifier)
    {
      original_mgm.add_outgoing_connection(new_connection_id);
    };
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

  pub fn set_depth_if_lower(&mut self, module_id: ModuleIdentifier, depth: usize) -> bool {
    let mgm = self
      .module_graph_module_by_identifier_mut(&module_id)
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
    if let Entry::Vacant(val) = self.module_identifier_to_module.entry(module.identifier()) {
      val.insert(module);
    }
  }

  pub fn add_block(&mut self, block: AsyncDependenciesBlock) {
    self.blocks.insert(block.identifier(), block);
  }

  pub fn set_parents(&mut self, dependency: DependencyId, parents: DependencyParents) {
    self.dependency_id_to_parents.insert(dependency, parents);
  }

  pub fn get_parent_module(&self, dependency: &DependencyId) -> Option<&ModuleIdentifier> {
    self
      .dependency_id_to_parents
      .get(dependency)
      .map(|p| &p.module)
  }

  pub fn get_parent_block(
    &self,
    dependency: &DependencyId,
  ) -> Option<&AsyncDependenciesBlockIdentifier> {
    self
      .dependency_id_to_parents
      .get(dependency)
      .and_then(|p| p.block.as_ref())
  }

  pub fn block_by_id(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> Option<&AsyncDependenciesBlock> {
    self.blocks.get(block_id)
  }

  pub fn dependencies(&self) -> &HashMap<DependencyId, BoxDependency> {
    &self.dependencies
  }

  pub fn add_dependency(&mut self, dependency: BoxDependency) {
    self.dependencies.insert(*dependency.id(), dependency);
  }

  pub fn dependency_by_id(&self, dependency_id: &DependencyId) -> Option<&BoxDependency> {
    self.dependencies.get(dependency_id)
  }

  fn remove_dependency(&mut self, dependency_id: &DependencyId) {
    self.dependencies.remove(dependency_id);
  }

  /// Uniquely identify a module by its dependency
  pub fn module_graph_module_by_dependency_id(
    &self,
    id: &DependencyId,
  ) -> Option<&ModuleGraphModule> {
    self
      .module_identifier_by_dependency_id(id)
      .and_then(|module_identifier| {
        self
          .module_identifier_to_module_graph_module
          .get(module_identifier)
      })
  }

  pub fn module_identifier_by_dependency_id(&self, id: &DependencyId) -> Option<&ModuleIdentifier> {
    self.dependency_id_to_module_identifier.get(id)
  }

  pub fn get_module(&self, dependency_id: &DependencyId) -> Option<&BoxModule> {
    let connection = self.connection_by_dependency(dependency_id)?;
    self.module_by_identifier(&connection.module_identifier)
  }

  /// Add a connection between two module graph modules, if a connection exists, then it will be reused.
  pub fn set_resolved_module(
    &mut self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let dependency = dependency_id.get_dependency(self);
    let is_module_dependency =
      dependency.as_module_dependency().is_some() || dependency.as_context_dependency().is_some();
    let condition = if self.is_new_treeshaking {
      dependency
        .as_module_dependency()
        .and_then(|dep| dep.get_condition())
    } else {
      None
    };
    self
      .dependency_id_to_module_identifier
      .insert(dependency_id, module_identifier);
    if !is_module_dependency {
      return Ok(());
    }

    let active = !matches!(condition, Some(DependencyCondition::False));
    let conditional = condition.is_some();
    // TODO: just a placeholder here, finish this when we have basic `getCondition` logic
    let new_connection = ModuleGraphConnection::new(
      original_module_identifier,
      dependency_id,
      module_identifier,
      active,
      conditional,
    );

    let connection_id = if let Some(connection_id) = self.connections_map.get(&new_connection) {
      *connection_id
    } else {
      let new_connection_id = ConnectionId::from(self.connections.len());
      self.connections.push(Some(new_connection));
      self
        .connections_map
        .insert(new_connection, new_connection_id);
      new_connection_id
    };
    if let Some(condition) = condition {
      self
        .connection_to_condition
        .insert(new_connection, condition);
    }

    self
      .dependency_id_to_connection_id
      .insert(dependency_id, connection_id);

    self
      .connection_id_to_dependency_id
      .insert(connection_id, dependency_id);

    {
      let mgm = self
        .module_graph_module_by_identifier_mut(&module_identifier)
        .unwrap_or_else(|| {
          panic!(
            "Failed to set resolved module: Module linked to module identifier {module_identifier} cannot be found"
          )
        });

      mgm.add_incoming_connection(connection_id);
    }

    if let Some(identifier) = original_module_identifier
      && let Some(original_mgm) = self.module_graph_module_by_identifier_mut(&identifier)
    {
      original_mgm.add_outgoing_connection(connection_id);
    };

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  #[inline]
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self.module_identifier_to_module.get(identifier)
  }

  /// Aggregate function which combine `get_normal_module_by_identifier`, `as_normal_module`, `get_resource_resolved_data`
  #[inline]
  pub fn normal_module_source_path_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<Cow<str>> {
    self
      .module_identifier_to_module
      .get(identifier)
      .and_then(|module| module.as_normal_module())
      .map(|module| {
        module
          .resource_resolved_data()
          .resource_path
          .to_string_lossy()
      })
  }

  /// Uniquely identify a module by its identifier and return the exclusive reference
  #[inline]
  pub fn module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut BoxModule> {
    self.module_identifier_to_module.get_mut(identifier)
  }

  #[inline]
  pub fn connection_id_by_dependency_id(&self, dep_id: &DependencyId) -> Option<&ConnectionId> {
    self.dependency_id_to_connection_id.get(dep_id)
  }
  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  #[inline]
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<&ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the exclusive reference
  #[inline]
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get_mut(identifier)
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
    self
      .dependency_id_to_connection_id
      .get(dependency_id)
      .and_then(|connection_id| self.connection_by_connection_id(connection_id))
  }

  pub fn connection_by_dependency_mut(
    &mut self,
    dependency_id: &DependencyId,
  ) -> Option<&mut ModuleGraphConnection> {
    self
      .dependency_id_to_connection_id
      .get(dependency_id)
      .cloned()
      .and_then(|connection_id| self.connection_by_connection_id_mut(&connection_id))
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

  pub fn dependency_by_connection_id(
    &self,
    connection_id: &ConnectionId,
  ) -> Option<&BoxDependency> {
    self
      .connection_id_to_dependency_id
      .get(connection_id)
      .and_then(|dependency_id| self.dependency_by_id(dependency_id))
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
    self.connections[**connection_id].as_ref()
  }

  pub fn connection_by_connection_id_mut(
    &mut self,
    connection_id: &ConnectionId,
  ) -> Option<&mut ModuleGraphConnection> {
    self.connections[**connection_id].as_mut()
  }

  pub fn remove_connection_by_dependency(
    &mut self,
    dependency_id: &DependencyId,
  ) -> Option<ModuleGraphConnection> {
    let mut removed = None;

    if let Some(connection_id) = self.dependency_id_to_connection_id.remove(dependency_id) {
      self.connection_id_to_dependency_id.remove(&connection_id);

      if let Some(connection) = self.connections[*connection_id].take() {
        self.connections_map.remove(&connection);

        if let Some(mgm) = connection
          .original_module_identifier
          .as_ref()
          .and_then(|ident| self.module_graph_module_by_identifier_mut(ident))
        {
          mgm.remove_outgoing_connection(connection_id);
        };

        if let Some(mgm) = self.module_graph_module_by_identifier_mut(&connection.module_identifier)
        {
          mgm.remove_incoming_connection(connection_id);
        }

        removed = Some(connection);
      }
    }
    self
      .dependency_id_to_module_identifier
      .remove(dependency_id);
    self.remove_dependency(dependency_id);

    removed
  }

  pub fn get_pre_order_index(&self, module_identifier: &ModuleIdentifier) -> Option<u32> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.pre_order_index)
  }

  pub fn get_issuer(&self, module: &BoxModule) -> Option<&BoxModule> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .and_then(|mgm| mgm.get_issuer().get_module(self))
  }

  pub fn is_async(&self, module: &ModuleIdentifier) -> Option<bool> {
    self
      .module_graph_module_by_identifier(module)
      .map(|mgm| mgm.is_async)
  }

  pub fn set_async(&mut self, module: &ModuleIdentifier) {
    if let Some(mgm) = self.module_graph_module_by_identifier_mut(module) {
      mgm.is_async = true
    }
  }

  pub fn get_outgoing_connections(&self, module: &BoxModule) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .map(|mgm| {
        mgm
          .outgoing_connections()
          .iter()
          .filter_map(|id| self.connection_by_connection_id(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_outgoing_connections_by_identifier(
    &self,
    module: &ModuleIdentifier,
  ) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(module)
      .map(|mgm| {
        mgm
          .outgoing_connections()
          .iter()
          .filter_map(|id| self.connection_by_connection_id(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_incoming_connections(&self, module: &BoxModule) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .map(|mgm| {
        mgm
          .incoming_connections()
          .iter()
          .filter_map(|id| self.connection_by_connection_id(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_incoming_connections_cloned(
    &self,
    module: &BoxModule,
  ) -> HashSet<ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .map(|mgm| {
        mgm
          .incoming_connections()
          .clone()
          .into_iter()
          .filter_map(|id| self.connection_by_connection_id(&id).cloned())
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_profile(&self, module: &BoxModule) -> Option<&ModuleProfile> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .and_then(|mgm| mgm.get_profile())
  }

  /// Remove a connection and return connection origin module identifier and dependency
  fn revoke_connection(&mut self, connection_id: ConnectionId) -> Option<BuildDependency> {
    let connection = match self.connections[*connection_id].take() {
      Some(c) => c,
      None => return None,
    };
    self.connections_map.remove(&connection);

    let ModuleGraphConnection {
      original_module_identifier,
      module_identifier,
      dependency_id,
      active,
      ..
    } = connection;
    // remove active connection id if current connection not active
    if !active
      && let Some(active_connection_id) = self.dependency_id_to_connection_id.get(&dependency_id)
      && active_connection_id != &connection_id
    {
      self.revoke_connection(*active_connection_id);
    }

    // remove dependency
    self.dependency_id_to_connection_id.remove(&dependency_id);
    self
      .dependency_id_to_module_identifier
      .remove(&dependency_id);
    self.connection_id_to_dependency_id.remove(&connection_id);

    // remove outgoing from original module graph module
    if let Some(original_module_identifier) = &original_module_identifier {
      if let Some(mgm) = self
        .module_identifier_to_module_graph_module
        .get_mut(original_module_identifier)
      {
        mgm.remove_outgoing_connection(connection_id);
        // Because of mgm.dependencies is set when original module build success
        // it does not need to remove dependency in mgm.dependencies.
      }
    }
    // remove incoming from module graph module
    if let Some(mgm) = self
      .module_identifier_to_module_graph_module
      .get_mut(&module_identifier)
    {
      mgm.remove_incoming_connection(connection_id);
    }

    Some((dependency_id, original_module_identifier))
  }

  /// Remove module from module graph and return parent module identifier and dependency pair
  pub fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    self.module_identifier_to_module.remove(module_identifier);
    let mgm = self
      .module_identifier_to_module_graph_module
      .remove(module_identifier);

    if let Some(mgm) = mgm {
      for cid in mgm.outgoing_connections() {
        self.revoke_connection(*cid);
      }

      mgm
        .incoming_connections()
        .iter()
        .filter_map(|cid| self.revoke_connection(*cid))
        .collect()
    } else {
      vec![]
    }
  }

  pub fn set_module_build_info_and_meta(
    &mut self,
    module_identifier: &ModuleIdentifier,
    build_info: BuildInfo,
    build_meta: BuildMeta,
  ) {
    if let Some(module) = self.module_by_identifier_mut(module_identifier) {
      module.set_module_build_info_and_meta(build_info, build_meta);
    }
  }

  #[inline]
  pub fn get_module_hash(&self, module_identifier: &ModuleIdentifier) -> Option<&RspackHashDigest> {
    self
      .module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.build_info().as_ref().and_then(|i| i.hash.as_ref()))
  }

  pub fn is_module_invalidated(
    &self,
    module_identifier: &ModuleIdentifier,
    files: &HashSet<PathBuf>,
  ) -> bool {
    if let Some(build_info) = self
      .module_by_identifier(module_identifier)
      .and_then(|module| module.build_info())
    {
      if !build_info.cacheable {
        return true;
      }

      for item in files {
        if build_info.file_dependencies.contains(item)
          || build_info.build_dependencies.contains(item)
          || build_info.context_dependencies.contains(item)
          || build_info.missing_dependencies.contains(item)
        {
          return true;
        }
      }
    }

    false
  }

  /// We can't insert all sort of things into one hashmap like javascript, so we create different
  /// hashmap to store different kinds of meta.
  pub fn get_dep_meta_if_existing(&self, id: DependencyId) -> Option<&DependencyExtraMeta> {
    self.dep_meta_map.get(&id)
  }

  pub fn update_module(&mut self, dep_id: &DependencyId, module_id: &ModuleIdentifier) {
    let connection_id = *self
      .connection_id_by_dependency_id(dep_id)
      .expect("should have connection id");
    let connection = self
      .connection_by_connection_id_mut(&connection_id)
      .expect("should have connection");
    if &connection.module_identifier == module_id {
      return;
    }

    // clone connection
    let mut new_connection = *connection;
    new_connection.module_identifier = *module_id;
    // modify connection
    connection.set_active(false);
    let connection = *connection;

    let new_connection_id = ConnectionId::from(self.connections.len());
    self.connections.push(Some(new_connection));
    self
      .connections_map
      .insert(new_connection, new_connection_id);

    // link dependency to new connection
    let old_connection_dependency_id = connection.dependency_id;
    self.dependency_id_to_module_identifier.insert(
      old_connection_dependency_id,
      new_connection.module_identifier,
    );
    self
      .dependency_id_to_connection_id
      .insert(old_connection_dependency_id, new_connection_id);
    self
      .connection_id_to_dependency_id
      .insert(new_connection_id, old_connection_dependency_id);

    // add new connection to original_module outgoing connections
    if let Some(original_module_identifier) = &new_connection.original_module_identifier {
      if let Some(mgm) = self.module_graph_module_by_identifier_mut(original_module_identifier) {
        mgm.add_outgoing_connection(new_connection_id);
        mgm.remove_outgoing_connection(connection_id);
      }
    }
    // add new connection to module incoming connections
    if let Some(mgm) = self.module_graph_module_by_identifier_mut(&new_connection.module_identifier)
    {
      mgm.add_incoming_connection(new_connection_id);
      mgm.remove_incoming_connection(connection_id);
    }

    // copy condition
    let condition = self.connection_to_condition.get(&connection);
    if let Some(condition) = condition {
      self
        .connection_to_condition
        .insert(new_connection, condition.clone());
    }
  }

  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> &ExportsInfo {
    let mgm = self
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    let exports_info = self.exports_info_map.get(*mgm.exports as usize);
    exports_info
  }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfoId) -> &ExportsInfo {
    let exports_info = self.exports_info_map.get((**id) as usize);
    exports_info
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfoId) -> &mut ExportsInfo {
    let exports_info = self.exports_info_map.get_mut((**id) as usize);
    exports_info
  }

  pub fn get_export_info_by_id(&self, id: &ExportInfoId) -> &ExportInfo {
    let export_info = self.export_info_map.get(**id as usize);
    export_info
  }

  pub fn get_export_info_mut_by_id(&mut self, id: &ExportInfoId) -> &mut ExportInfo {
    let exports_info = self.export_info_map.get_mut(**id as usize);

    exports_info
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
    module_id: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    let mgm = self
      .module_graph_module_by_identifier(&module_id)
      .expect("should have module graph module");
    mgm
      .exports
      .get_exports_info(self)
      .get_used_exports(self, runtime)
  }

  pub fn get_optimization_bailout_mut(&mut self, module: ModuleIdentifier) -> &mut Vec<String> {
    let mgm = self
      .module_graph_module_by_identifier_mut(&module)
      .expect("should have module graph module");
    mgm.optimization_bailout_mut()
  }

  pub fn get_read_only_export_info(
    &self,
    module_identifier: &ModuleIdentifier,
    name: Atom,
  ) -> Option<&ExportInfo> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|mgm| mgm.exports.get_read_only_export_info(&name, self))
  }
}

fn get_connections_by_origin_module(
  connections: impl Iterator<Item = &ConnectionId>,
  mg: &ModuleGraph,
) -> HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> {
  let mut map: HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> = HashMap::default();

  for connection_id in connections {
    let con = mg
      .connection_by_connection_id(connection_id)
      .expect("should have connection");
    match map.entry(con.original_module_identifier) {
      Entry::Occupied(mut occ) => {
        occ.get_mut().push(*con);
      }
      Entry::Vacant(vac) => {
        vac.insert(vec![*con]);
      }
    }
  }

  map
}

#[cfg(test)]
mod test {
  use std::borrow::Cow;

  use rspack_error::{Diagnosable, Diagnostic, Result};
  use rspack_identifier::Identifiable;
  use rspack_sources::Source;
  use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

  use crate::{
    AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta,
    BuildResult, CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock,
    Dependency, DependencyId, ExportInfo, ExportsInfo, Module, ModuleDependency, ModuleGraph,
    ModuleGraphModule, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType, UsageState,
  };

  // Define a detailed node type for `ModuleGraphModule`s
  #[derive(Debug, PartialEq, Eq, Hash)]
  struct Node(&'static str);

  macro_rules! impl_noop_trait_module_type {
    ($ident:ident) => {
      impl Identifiable for $ident {
        fn identifier(&self) -> ModuleIdentifier {
          (stringify!($ident).to_owned() + "__" + self.0).into()
        }
      }

      impl Diagnosable for $ident {}

      impl DependenciesBlock for $ident {
        fn add_block_id(&mut self, _: AsyncDependenciesBlockIdentifier) {
          unreachable!()
        }

        fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
          unreachable!()
        }

        fn add_dependency_id(&mut self, _: DependencyId) {
          unreachable!()
        }

        fn get_dependencies(&self) -> &[DependencyId] {
          unreachable!()
        }
      }

      #[::async_trait::async_trait]
      impl Module for $ident {
        fn module_type(&self) -> &ModuleType {
          unreachable!()
        }

        fn source_types(&self) -> &[SourceType] {
          unreachable!()
        }

        fn original_source(&self) -> Option<&dyn Source> {
          unreachable!()
        }

        fn size(&self, _source_type: &SourceType) -> f64 {
          unreachable!()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<str> {
          unreachable!()
        }

        fn get_diagnostics(&self) -> Vec<Diagnostic> {
          vec![]
        }

        async fn build(
          &mut self,
          _build_context: BuildContext<'_>,
          _compilation: Option<&Compilation>,
        ) -> Result<BuildResult> {
          unreachable!()
        }

        fn code_generation(
          &self,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
          _concatenation_scope: Option<ConcatenationScope>,
        ) -> Result<CodeGenerationResult> {
          unreachable!()
        }

        fn build_meta(&self) -> Option<&BuildMeta> {
          unreachable!()
        }

        fn build_info(&self) -> Option<&BuildInfo> {
          unreachable!()
        }

        fn set_module_build_info_and_meta(
          &mut self,
          _build_info: BuildInfo,
          _build_meta: BuildMeta,
        ) {
          unreachable!()
        }
      }

      impl ModuleSourceMapConfig for $ident {
        fn get_source_map_kind(&self) -> &SourceMapKind {
          unreachable!()
        }
        fn set_source_map_kind(&mut self, _source_map: SourceMapKind) {
          unreachable!()
        }
      }
    };
  }

  impl_noop_trait_module_type!(Node);

  // Define a detailed edge type for `ModuleGraphConnection`s, tuple contains the parent module identifier and the child module specifier
  #[derive(Debug, PartialEq, Eq, Hash, Clone)]
  struct Edge(Option<ModuleIdentifier>, String, DependencyId);

  macro_rules! impl_noop_trait_dep_type {
    ($ident:ident) => {
      impl Dependency for $ident {
        fn id(&self) -> &DependencyId {
          &self.2
        }
        fn dependency_debug_name(&self) -> &'static str {
          stringify!($ident)
        }
      }

      impl ModuleDependency for $ident {
        fn request(&self) -> &str {
          &*self.1
        }

        fn user_request(&self) -> &str {
          &*self.1
        }

        fn set_request(&mut self, request: String) {
          self.1 = request;
        }
      }

      impl crate::AsDependencyTemplate for $ident {}
      impl crate::AsContextDependency for $ident {}
    };
  }

  impl_noop_trait_dep_type!(Edge);

  fn add_module_to_graph(mg: &mut ModuleGraph, m: Box<dyn Module>) {
    let other_exports_info = ExportInfo::new(None, UsageState::Unknown, None);
    let side_effects_only_info = ExportInfo::new(
      Some("*side effects only*".into()),
      UsageState::Unknown,
      None,
    );
    let exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
    let mgm = ModuleGraphModule::new(m.identifier(), ModuleType::Js, exports_info.id);
    mg.add_module_graph_module(mgm);
    mg.add_module(m);
    mg.export_info_map
      .insert(*other_exports_info.id as usize, other_exports_info);
    mg.export_info_map
      .insert(*side_effects_only_info.id as usize, side_effects_only_info);
    mg.exports_info_map
      .insert(*exports_info.id as usize, exports_info);
  }

  fn link_modules_with_dependency(
    mg: &mut ModuleGraph,
    from: Option<&ModuleIdentifier>,
    to: &ModuleIdentifier,
    dep: BoxDependency,
  ) -> DependencyId {
    let dependency_id = *dep.id();
    mg.add_dependency(dep);
    mg.dependency_id_to_module_identifier
      .insert(dependency_id, *to);
    if let Some(p_id) = from
      && let Some(mgm) = mg.module_graph_module_by_identifier_mut(p_id)
    {
      mgm.__deprecated_all_dependencies.push(dependency_id);
    }
    mg.set_resolved_module(from.copied(), dependency_id, *to)
      .expect("failed to set resolved module");

    assert_eq!(
      mg.dependency_id_to_module_identifier
        .get(&dependency_id)
        .copied(),
      Some(*to)
    );
    dependency_id
  }

  fn mgm<'m>(mg: &'m ModuleGraph, m_id: &ModuleIdentifier) -> &'m ModuleGraphModule {
    mg.module_graph_module_by_identifier(m_id)
      .expect("not found")
  }

  macro_rules! node {
    ($s:literal) => {
      Node($s)
    };
  }

  macro_rules! edge {
    ($from:literal, $to:expr) => {
      Edge(Some($from.into()), $to.into(), DependencyId::new())
    };
    ($from:expr, $to:expr) => {
      Edge($from, $to.into(), DependencyId::new())
    };
  }

  #[test]
  fn test_module_graph() {
    let mut mg = ModuleGraph::default();
    let a = node!("a");
    let b = node!("b");
    let a_id = a.identifier();
    let b_id = b.identifier();
    let a_to_b = edge!(Some(a_id), b_id.as_str());
    add_module_to_graph(&mut mg, Box::new(a));
    add_module_to_graph(&mut mg, Box::new(b));
    let a_to_b_id = link_modules_with_dependency(&mut mg, Some(&a_id), &b_id, Box::new(a_to_b));

    let mgm_a = mgm(&mg, &a_id);
    let mgm_b = mgm(&mg, &b_id);
    let conn_a = mgm_a.outgoing_connections().iter().collect::<Vec<_>>();
    let conn_b = mgm_b.incoming_connections().iter().collect::<Vec<_>>();
    assert_eq!(conn_a[0], conn_b[0]);

    let c = node!("c");
    let c_id = c.identifier();
    let b_to_c = edge!(Some(b_id), c_id.as_str());
    add_module_to_graph(&mut mg, Box::new(c));
    let b_to_c_id = link_modules_with_dependency(&mut mg, Some(&b_id), &c_id, Box::new(b_to_c));

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    let conn_b = mgm_b.outgoing_connections().iter().collect::<Vec<_>>();
    let conn_c = mgm_c.incoming_connections().iter().collect::<Vec<_>>();
    assert_eq!(conn_c[0], conn_b[0]);

    mg.remove_connection_by_dependency(&a_to_b_id);

    let mgm_a = mgm(&mg, &a_id);
    let mgm_b = mgm(&mg, &b_id);
    assert!(mgm_a.outgoing_connections().is_empty());
    assert!(mgm_b.incoming_connections().is_empty());

    mg.remove_connection_by_dependency(&b_to_c_id);

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    assert!(mgm_b.outgoing_connections().is_empty());
    assert!(mgm_c.incoming_connections().is_empty());
  }
}
