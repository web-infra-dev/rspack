use std::collections::hash_map::Entry;

use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::atoms::Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, Dependency,
  ExportProvided, ProvidedExports, RuntimeSpec, UsedExports,
};
mod module;
pub use module::*;
mod connection;
pub use connection::*;

use crate::{
  BoxDependency, BoxModule, DependencyCondition, DependencyId, ExportInfo, ExportInfoData,
  ExportsInfo, ExportsInfoData, ModuleIdentifier,
};

// TODO Here request can be used Atom
pub type ImportVarMap =
  HashMap<Option<ModuleIdentifier> /* request */, String /* import_var */>;

pub type BuildDependency = (
  DependencyId,
  Option<ModuleIdentifier>, /* parent module */
);

/// https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L742-L748
#[derive(Debug)]
pub struct DependencyExtraMeta {
  pub ids: Vec<Atom>,
  pub explanation: Option<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct DependencyParents {
  pub block: Option<AsyncDependenciesBlockIdentifier>,
  pub module: ModuleIdentifier,
  pub index_in_block: usize,
}

#[derive(Debug, Default)]
pub struct ModuleGraphPartial {
  /// Module indexed by `ModuleIdentifier`.
  pub(crate) modules: IdentifierMap<Option<BoxModule>>,

  /// Dependencies indexed by `DependencyId`.
  dependencies: HashMap<DependencyId, Option<BoxDependency>>,

  /// AsyncDependenciesBlocks indexed by `AsyncDependenciesBlockIdentifier`.
  blocks: HashMap<AsyncDependenciesBlockIdentifier, Option<Box<AsyncDependenciesBlock>>>,

  /// ModuleGraphModule indexed by `ModuleIdentifier`.
  module_graph_modules: IdentifierMap<Option<ModuleGraphModule>>,

  /// ModuleGraphConnection indexed by `DependencyId`.
  connections: HashMap<DependencyId, Option<ModuleGraphConnection>>,

  /// Dependency_id to parent module identifier and parent block
  ///
  /// # Example
  ///
  /// ```ignore
  /// let parent_module_id = parent_module.identifier();
  /// parent_module
  ///   .get_dependencies()
  ///   .iter()
  ///   .map(|dependency_id| {
  ///     let parents_info = module_graph_partial
  ///       .dependency_id_to_parents
  ///       .get(dependency_id)
  ///       .unwrap()
  ///       .unwrap();
  ///     assert_eq!(parents_info, parent_module_id);
  ///   })
  /// ```
  dependency_id_to_parents: HashMap<DependencyId, Option<DependencyParents>>,

  // Module's ExportsInfo is also a part of ModuleGraph
  exports_info_map: UkeyMap<ExportsInfo, ExportsInfoData>,
  export_info_map: UkeyMap<ExportInfo, ExportInfoData>,
  connection_to_condition: HashMap<DependencyId, DependencyCondition>,
  dep_meta_map: HashMap<DependencyId, DependencyExtraMeta>,
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

  /// Return an unordered iterator of modules
  pub fn modules(&self) -> IdentifierMap<&BoxModule> {
    let mut res = IdentifierMap::default();
    for item in self.partials.iter() {
      for (k, v) in &item.modules {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.modules {
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
      for (k, v) in &item.module_graph_modules {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.module_graph_modules {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    res
  }

  // #[tracing::instrument(skip_all, fields(module = ?module_id))]
  pub fn get_incoming_connections_by_origin_module(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> {
    let connections = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have mgm")
      .incoming_connections();

    let mut map: HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> = HashMap::default();
    for dep_id in connections {
      let con = self
        .connection_by_dependency_id(dep_id)
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

  /// Remove dependency in mgm and target connection, return dependency_id and origin module identifier
  ///
  /// force will completely remove dependency, and you will not regenerate it from dependency_id
  pub fn revoke_dependency(
    &mut self,
    dep_id: &DependencyId,
    force: bool,
  ) -> Option<BuildDependency> {
    let original_module_identifier = self.get_parent_module(dep_id).copied();
    let module_identifier = self.module_identifier_by_dependency_id(dep_id).copied();

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    if module_identifier.is_some() {
      active_partial.connections.insert(*dep_id, None);
    }
    if force {
      active_partial.dependencies.insert(*dep_id, None);
      active_partial
        .dependency_id_to_parents
        .insert(*dep_id, None);
    }

    // remove outgoing from original module graph module
    if let Some(original_module_identifier) = &original_module_identifier {
      if let Some(mgm) = self.module_graph_module_by_identifier_mut(original_module_identifier) {
        mgm.remove_outgoing_connection(dep_id);
        // Because of mgm.all_dependencies is set when original module build success
        // it does not need to remove dependency in mgm.all_dependencies.
      }
    }
    // remove incoming from module graph module
    if let Some(module_identifier) = &module_identifier {
      if let Some(mgm) = self.module_graph_module_by_identifier_mut(module_identifier) {
        mgm.remove_incoming_connection(dep_id);
      }
    }

    Some((*dep_id, original_module_identifier))
  }

  pub fn revoke_module(&mut self, module_id: &ModuleIdentifier) -> Vec<BuildDependency> {
    let blocks = self
      .module_by_identifier(module_id)
      .map(|m| Vec::from(m.get_blocks()))
      .unwrap_or_default();

    let (incoming_connections, all_dependencies) = self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        (
          mgm.incoming_connections().clone(),
          mgm.all_dependencies.clone(),
        )
      })
      .unwrap_or_default();

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };

    active_partial.modules.insert(*module_id, None);
    active_partial.module_graph_modules.insert(*module_id, None);

    for block in blocks {
      active_partial.blocks.insert(block, None);
    }

    for dep_id in all_dependencies {
      self.revoke_dependency(&dep_id, true);
    }

    incoming_connections
      .iter()
      .filter_map(|dep_id| self.revoke_dependency(dep_id, false))
      .collect()
  }

  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    match active_partial
      .module_graph_modules
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
    compilation: &mut Compilation,
    source_module: &ModuleIdentifier,
    target_module: &ModuleIdentifier,
  ) {
    let mut module_graph = compilation.get_module_graph_mut();
    let old_mgm = module_graph
      .module_graph_module_by_identifier(source_module)
      .expect("should have mgm");

    // Using this tuple to avoid violating rustc borrow rules
    let assign_tuple = (
      old_mgm.post_order_index,
      old_mgm.pre_order_index,
      old_mgm.depth,
      old_mgm.exports,
    );
    let new_mgm = module_graph
      .module_graph_module_by_identifier_mut(target_module)
      .expect("should have mgm");
    new_mgm.post_order_index = assign_tuple.0;
    new_mgm.pre_order_index = assign_tuple.1;
    new_mgm.depth = assign_tuple.2;
    new_mgm.exports = assign_tuple.3;

    let is_async = ModuleGraph::is_async(compilation, source_module);
    ModuleGraph::set_async(compilation, *target_module, is_async);
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

    // Outgoing connections
    let outgoing_connections = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm")
      .outgoing_connections()
      .clone();
    let mut affected_outgoing_connection = vec![];
    for dep_id in outgoing_connections {
      let connection = self
        .connection_by_dependency_id(&dep_id)
        .expect("should have connection");
      let dependency = self
        .dependency_by_id(&dep_id)
        .expect("should have dependency");
      if filter_connection(connection, dependency) {
        let connection = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        connection.original_module_identifier = Some(*new_module);
        affected_outgoing_connection.push(dep_id);
      }
    }

    let old_mgm = self
      .module_graph_module_by_identifier_mut(old_module)
      .expect("should have mgm");
    for dep_id in &affected_outgoing_connection {
      old_mgm.remove_outgoing_connection(dep_id);
    }

    let new_mgm = self
      .module_graph_module_by_identifier_mut(new_module)
      .expect("should have mgm");
    for dep_id in affected_outgoing_connection {
      new_mgm.add_outgoing_connection(dep_id);
    }

    // Incoming connections
    let incoming_connections = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm")
      .incoming_connections()
      .clone();
    let mut affected_incoming_connection = vec![];
    for dep_id in incoming_connections {
      let connection = self
        .connection_by_dependency_id(&dep_id)
        .expect("should have connection");
      let dependency = self
        .dependency_by_id(&dep_id)
        .expect("should have dependency");
      // the inactive connection should not be updated
      if filter_connection(connection, dependency) && (connection.conditional || connection.active)
      {
        let connection = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        connection.set_module_identifier(*new_module);
        affected_incoming_connection.push(dep_id);
      }
    }

    let old_mgm = self
      .module_graph_module_by_identifier_mut(old_module)
      .expect("should have mgm");
    for dep_id in &affected_incoming_connection {
      old_mgm.remove_incoming_connection(dep_id);
    }

    let new_mgm = self
      .module_graph_module_by_identifier_mut(new_module)
      .expect("should have mgm");
    for dep_id in affected_incoming_connection {
      new_mgm.add_incoming_connection(dep_id);
    }
  }

  pub fn copy_outgoing_module_connections<F>(
    &mut self,
    old_module: &ModuleIdentifier,
    new_module: &ModuleIdentifier,
    filter_connection: F,
  ) where
    F: Fn(&ModuleGraphConnection, &BoxDependency) -> bool,
  {
    if old_module == new_module {
      return;
    }

    let old_mgm_connections = self
      .module_graph_module_by_identifier(old_module)
      .expect("should have mgm")
      .outgoing_connections()
      .clone();

    // Outgoing connections
    let mut affected_outgoing_connections = vec![];
    for dep_id in old_mgm_connections {
      let connection = self
        .connection_by_dependency_id(&dep_id)
        .expect("should have connection");
      let dep = self
        .dependency_by_id(&dep_id)
        .expect("should have dependency");
      if filter_connection(connection, dep) {
        let con = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        con.original_module_identifier = Some(*new_module);
        affected_outgoing_connections.push(dep_id);
      }
    }

    let new_mgm = self
      .module_graph_module_by_identifier_mut(new_module)
      .expect("should have mgm");
    for dep_id in affected_outgoing_connections {
      new_mgm.add_outgoing_connection(dep_id);
    }
  }

  pub fn get_depth(&self, module_id: &ModuleIdentifier) -> Option<usize> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.depth)
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
    match active_partial.modules.entry(module.identifier()) {
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

  pub fn add_block(&mut self, block: Box<AsyncDependenciesBlock>) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .blocks
      .insert(block.identifier(), Some(block));
  }

  pub fn set_parents(&mut self, dependency_id: DependencyId, parents: DependencyParents) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial
      .dependency_id_to_parents
      .insert(dependency_id, Some(parents));
  }

  pub fn get_parent_module(&self, dependency_id: &DependencyId) -> Option<&ModuleIdentifier> {
    self
      .loop_partials(|p| p.dependency_id_to_parents.get(dependency_id))?
      .as_ref()
      .map(|p| &p.module)
  }

  pub fn get_parent_block(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<&AsyncDependenciesBlockIdentifier> {
    self
      .loop_partials(|p| p.dependency_id_to_parents.get(dependency_id))?
      .as_ref()
      .map(|p| &p.block)?
      .as_ref()
  }

  pub fn get_parent_block_index(&self, dependency_id: &DependencyId) -> Option<usize> {
    self
      .loop_partials(|p| p.dependency_id_to_parents.get(dependency_id))?
      .as_ref()
      .map(|p| p.index_in_block)
  }

  pub fn block_by_id(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> Option<&AsyncDependenciesBlock> {
    self
      .loop_partials(|p| p.blocks.get(block_id))?
      .as_ref()
      .map(|b| &**b)
  }

  pub fn block_by_id_mut(
    &mut self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> Option<&mut Box<AsyncDependenciesBlock>> {
    self
      .loop_partials_mut(
        |p| p.blocks.contains_key(block_id),
        |p, search_result| {
          p.blocks.insert(*block_id, search_result);
        },
        |p| p.blocks.get(block_id).cloned(),
        |p| p.blocks.get_mut(block_id),
      )?
      .as_mut()
  }

  pub fn block_by_id_expect(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> &AsyncDependenciesBlock {
    self
      .loop_partials(|p| p.blocks.get(block_id))
      .expect("should insert block before get it")
      .as_ref()
      .expect("block has been removed to None")
  }

  pub fn dependencies(&self) -> HashMap<DependencyId, &BoxDependency> {
    let mut res = HashMap::default();
    for item in self.partials.iter() {
      for (k, v) in &item.dependencies {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
      }
    }
    if let Some(active) = &self.active {
      for (k, v) in &active.dependencies {
        if let Some(v) = v {
          res.insert(*k, v);
        } else {
          res.remove(k);
        }
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
      .insert(*dependency.id(), Some(dependency));
  }

  pub fn dependency_by_id(&self, dependency_id: &DependencyId) -> Option<&BoxDependency> {
    self
      .loop_partials(|p| p.dependencies.get(dependency_id))?
      .as_ref()
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

  pub fn module_identifier_by_dependency_id(
    &self,
    dep_id: &DependencyId,
  ) -> Option<&ModuleIdentifier> {
    self
      .loop_partials(|p| p.connections.get(dep_id))?
      .as_ref()
      .map(|con| con.module_identifier())
  }

  pub fn get_module_by_dependency_id(&self, dep_id: &DependencyId) -> Option<&BoxModule> {
    if let Some(module_id) = self.module_identifier_by_dependency_id(dep_id) {
      self.loop_partials(|p| p.modules.get(module_id))?.as_ref()
    } else {
      None
    }
  }

  fn add_connection(
    &mut self,
    connection: ModuleGraphConnection,
    condition: Option<DependencyCondition>,
  ) {
    if self
      .connection_by_dependency_id(&connection.dependency_id)
      .is_some()
    {
      return;
    }

    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };

    if let Some(condition) = condition {
      active_partial
        .connection_to_condition
        .insert(connection.dependency_id, condition);
    }

    let module_id = *connection.module_identifier();
    let origin_module_id = connection.original_module_identifier;
    let dependency_id = connection.dependency_id;

    // add to connections list
    active_partial
      .connections
      .insert(connection.dependency_id, Some(connection));

    // set to module incoming connection
    {
      let mgm = self
        .module_graph_module_by_identifier_mut(&module_id)
        .unwrap_or_else(|| {
          panic!(
            "Failed to add connection: Module linked to module identifier {module_id} cannot be found"
          )
        });

      mgm.add_incoming_connection(dependency_id);
    }

    // set to origin module outgoing connection
    if let Some(identifier) = origin_module_id
      && let Some(original_mgm) = self.module_graph_module_by_identifier_mut(&identifier)
    {
      original_mgm.add_outgoing_connection(dependency_id);
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
    let condition = dependency
      .as_module_dependency()
      .and_then(|dep| dep.get_condition());
    if !is_module_dependency {
      return Ok(());
    }

    let active = !matches!(condition, Some(DependencyCondition::False));
    let conditional = condition.is_some();
    let new_connection = ModuleGraphConnection::new(
      dependency_id,
      original_module_identifier,
      module_identifier,
      active,
      conditional,
    );
    self.add_connection(new_connection, condition);

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self.loop_partials(|p| p.modules.get(identifier))?.as_ref()
  }

  pub fn module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut BoxModule> {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    if let Some(res) = active_partial.modules.get_mut(identifier) {
      res.as_mut()
    } else {
      None
    }
  }

  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<&ModuleGraphModule> {
    self
      .loop_partials(|p| p.module_graph_modules.get(identifier))?
      .as_ref()
  }

  /// Uniquely identify a module graph module by its module's identifier and return the exclusive reference
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut ModuleGraphModule> {
    self
      .loop_partials_mut(
        |p| p.module_graph_modules.contains_key(identifier),
        |p, search_result| {
          p.module_graph_modules.insert(*identifier, search_result);
        },
        |p| p.module_graph_modules.get(identifier).cloned(),
        |p| p.module_graph_modules.get_mut(identifier),
      )?
      .as_mut()
  }

  /// refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L582-L585
  pub fn get_export_info(&mut self, module_id: ModuleIdentifier, export_name: &Atom) -> ExportInfo {
    let exports_info = self.get_exports_info(&module_id);
    exports_info.get_export_info(self, export_name)
  }

  pub fn get_outgoing_connections_in_order(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> impl Iterator<Item = &DependencyId> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|m| {
        m.all_dependencies
          .iter()
          .filter(|dep_id| self.connection_by_dependency_id(dep_id).is_some())
      })
      .into_iter()
      .flatten()
  }

  pub fn connection_by_dependency_id(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<&ModuleGraphConnection> {
    self
      .loop_partials(|p| p.connections.get(dependency_id))?
      .as_ref()
  }

  pub fn connection_by_dependency_id_mut(
    &mut self,
    dependency_id: &DependencyId,
  ) -> Option<&mut ModuleGraphConnection> {
    self
      .loop_partials_mut(
        |p| p.connections.contains_key(dependency_id),
        |p, search_result| {
          p.connections.insert(*dependency_id, search_result);
        },
        |p| p.connections.get(dependency_id).cloned(),
        |p| p.connections.get_mut(dependency_id),
      )?
      .as_mut()
  }

  pub fn get_pre_order_index(&self, module_id: &ModuleIdentifier) -> Option<u32> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.pre_order_index)
  }

  pub fn get_post_order_index(&self, module_id: &ModuleIdentifier) -> Option<u32> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.post_order_index)
  }

  pub fn get_issuer(&self, module_id: &ModuleIdentifier) -> Option<&BoxModule> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.issuer().get_module(self))
  }

  pub fn is_optional(&self, module_id: &ModuleIdentifier) -> bool {
    let mut has_connections = false;
    for connection in self.get_incoming_connections(module_id) {
      let Some(dependency) = self
        .dependency_by_id(&connection.dependency_id)
        .and_then(|dep| dep.as_module_dependency())
      else {
        return false;
      };
      if !dependency.get_optional() || !connection.is_target_active(self, None) {
        return false;
      }
      has_connections = true;
    }
    has_connections
  }

  pub fn is_async(compilation: &Compilation, module_id: &ModuleIdentifier) -> bool {
    compilation.async_modules_artifact.contains(module_id)
  }

  pub fn set_async(
    compilation: &mut Compilation,
    module_id: ModuleIdentifier,
    is_async: bool,
  ) -> bool {
    let original = Self::is_async(compilation, &module_id);
    if original == is_async {
      return false;
    }
    if original {
      compilation.async_modules_artifact.remove(&module_id)
    } else {
      compilation.async_modules_artifact.insert(module_id)
    }
  }

  pub fn get_outgoing_connections(
    &self,
    module_id: &ModuleIdentifier,
  ) -> impl Iterator<Item = &ModuleGraphConnection> + Clone {
    self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        mgm
          .outgoing_connections()
          .iter()
          .filter_map(|id| self.connection_by_dependency_id(id))
      })
      .into_iter()
      .flatten()
  }

  pub fn get_incoming_connections(
    &self,
    module_id: &ModuleIdentifier,
  ) -> impl Iterator<Item = &ModuleGraphConnection> + Clone {
    self
      .module_graph_module_by_identifier(module_id)
      .map(|mgm| {
        mgm
          .incoming_connections()
          .iter()
          .filter_map(|id| self.connection_by_dependency_id(id))
      })
      .into_iter()
      .flatten()
  }

  pub fn get_module_hash(&self, module_id: &ModuleIdentifier) -> Option<&RspackHashDigest> {
    self
      .module_by_identifier(module_id)
      .and_then(|m| m.build_info().hash.as_ref())
  }

  /// We can't insert all sort of things into one hashmap like javascript, so we create different
  /// hashmap to store different kinds of meta.
  pub fn get_dep_meta_if_existing(&self, id: &DependencyId) -> Option<&DependencyExtraMeta> {
    self.loop_partials(|p| p.dep_meta_map.get(id))
  }

  pub fn set_dependency_extra_meta(&mut self, dep_id: DependencyId, extra: DependencyExtraMeta) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.dep_meta_map.insert(dep_id, extra);
  }

  pub fn can_update_module(&self, dep_id: &DependencyId, module_id: &ModuleIdentifier) -> bool {
    let connection = self
      .connection_by_dependency_id(dep_id)
      .expect("should have connection");
    connection.module_identifier() != module_id
  }

  pub fn do_update_module(&mut self, dep_id: &DependencyId, module_id: &ModuleIdentifier) {
    let connection = self
      .connection_by_dependency_id_mut(dep_id)
      .unwrap_or_else(|| panic!("{dep_id:?}"));
    let old_module_identifier = *connection.module_identifier();
    connection.set_module_identifier(*module_id);

    // remove dep_id from old module mgm incoming connection
    let old_mgm = self
      .module_graph_module_by_identifier_mut(&old_module_identifier)
      .expect("should exist mgm");
    old_mgm.remove_incoming_connection(dep_id);

    // add dep_id to updated module mgm incoming connection
    let new_mgm = self
      .module_graph_module_by_identifier_mut(module_id)
      .expect("should exist mgm");
    new_mgm.add_incoming_connection(*dep_id);
  }

  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    let mgm = self
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    self
      .loop_partials(|p| p.exports_info_map.get(&mgm.exports))
      .expect("should have exports info")
      .id()
  }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .try_get_exports_info_by_id(id)
      .expect("should have exports info")
  }

  pub fn try_get_exports_info_by_id(&self, id: &ExportsInfo) -> Option<&ExportsInfoData> {
    self.loop_partials(|p| p.exports_info_map.get(id))
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfo) -> &mut ExportsInfoData {
    self
      .loop_partials_mut(
        |p| p.exports_info_map.contains_key(id),
        |p, search_result| {
          p.exports_info_map.insert(*id, search_result);
        },
        |p| p.exports_info_map.get(id).cloned(),
        |p| p.exports_info_map.get_mut(id),
      )
      .expect("should have exports info")
  }

  pub fn set_exports_info(&mut self, id: ExportsInfo, info: ExportsInfoData) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.exports_info_map.insert(id, info);
  }

  pub fn try_get_export_info_by_id(&self, id: &ExportInfo) -> Option<&ExportInfoData> {
    self.loop_partials(|p| p.export_info_map.get(id))
  }

  pub fn get_export_info_by_id(&self, id: &ExportInfo) -> &ExportInfoData {
    self
      .try_get_export_info_by_id(id)
      .expect("should have export info")
  }

  pub fn get_export_info_mut_by_id(&mut self, id: &ExportInfo) -> &mut ExportInfoData {
    self
      .loop_partials_mut(
        |p| p.export_info_map.contains_key(id),
        |p, search_result| {
          p.export_info_map.insert(*id, search_result);
        },
        |p| p.export_info_map.get(id).cloned(),
        |p| p.export_info_map.get_mut(id),
      )
      .expect("should have export info")
  }

  pub fn set_export_info(&mut self, id: ExportInfo, info: ExportInfoData) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.export_info_map.insert(id, info);
  }

  pub fn get_provided_exports(&self, module_id: ModuleIdentifier) -> ProvidedExports {
    let mgm = self
      .module_graph_module_by_identifier(&module_id)
      .expect("should have module graph module");
    mgm.exports.get_provided_exports(self)
  }

  pub fn get_used_exports(
    &self,
    id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    let mgm = self
      .module_graph_module_by_identifier(id)
      .expect("should have module graph module");
    mgm.exports.get_used_exports(self, runtime)
  }

  pub fn get_optimization_bailout_mut(&mut self, id: &ModuleIdentifier) -> &mut Vec<String> {
    let mgm = self
      .module_graph_module_by_identifier_mut(id)
      .expect("should have module graph module");
    mgm.optimization_bailout_mut()
  }

  pub fn get_optimization_bailout(&self, id: &ModuleIdentifier) -> &Vec<String> {
    let mgm = self
      .module_graph_module_by_identifier(id)
      .expect("should have module graph module");
    &mgm.optimization_bailout
  }

  pub fn get_read_only_export_info(&self, id: &ModuleIdentifier, name: Atom) -> Option<ExportInfo> {
    self
      .module_graph_module_by_identifier(id)
      .map(|mgm| mgm.exports.get_read_only_export_info(self, &name))
  }

  pub fn get_condition_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
  ) -> ConnectionState {
    let condition = self
      .loop_partials(|p| p.connection_to_condition.get(&connection.dependency_id))
      .expect("should have condition");
    match condition {
      DependencyCondition::False => ConnectionState::Bool(false),
      DependencyCondition::Fn(f) => f.get_connection_state(connection, runtime, self),
    }
  }

  // returns: Option<bool>
  //   - None: it's unknown
  //   - Some(true): provided
  //   - Some(false): not provided
  pub fn is_export_provided(&self, id: &ModuleIdentifier, names: &[Atom]) -> Option<bool> {
    self.module_graph_module_by_identifier(id).and_then(|mgm| {
      match mgm.exports.is_export_provided(self, names)? {
        ExportProvided::True => Some(true),
        ExportProvided::False => Some(false),
        ExportProvided::Null => None,
      }
    })
  }

  // todo remove it after module_graph_partial remove all of dependency_id_to_*
  pub fn cache_recovery_connection(&mut self, connection: ModuleGraphConnection) {
    let condition = self
      .dependency_by_id(&connection.dependency_id)
      .and_then(|d| d.as_module_dependency())
      .and_then(|dep| dep.get_condition());
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };

    // recovery condition
    if let Some(condition) = condition {
      active_partial
        .connection_to_condition
        .insert(connection.dependency_id, condition);
    }

    active_partial
      .connections
      .insert(connection.dependency_id, Some(connection));
  }
}
