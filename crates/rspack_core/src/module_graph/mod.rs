pub mod internal;
pub mod rollback;

use internal::try_get_module_graph_module_mut_by_identifier;
use rayon::prelude::*;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::atoms::Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, AsyncModulesArtifact, Compilation,
  DependenciesBlock, Dependency, ExportInfo, ExportName, ImportedByDeferModulesArtifact,
  ModuleGraphCacheArtifact, RuntimeSpec, UsedNameItem,
};
mod module;
pub use module::*;
mod connection;
pub use connection::*;

use crate::{
  BoxDependency, BoxModule, DependencyCondition, DependencyId, ExportsInfoArtifact,
  ModuleIdentifier,
};

// TODO Here request can be used Atom
pub type ImportVarMap = HashMap<(Option<ModuleIdentifier>, bool), String /* import_var */>;

pub type BuildDependency = (
  DependencyId,
  Option<ModuleIdentifier>, /* parent module */
);

/// https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L742-L748
#[derive(Debug, Clone)]
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

/// Internal data structure for ModuleGraph
/// There're 3 kinds of data in module_graph here
/// 1. Data only setting during Make Phase which no need for clone or overlay to recover
/// 2. Data modified during Seal Phase which no need for clone or overlay to recover
/// 3. Data modified both during Seal Phase and Make Phase which need for clone or overlay to recover
///    3.1 only the contained modified in seal phase which can be reverted
///    3.2 the item is modified which can only use overlay or clone to recover
#[derive(Debug, Default)]
pub(crate) struct ModuleGraphData {
  /****** only modified during Make Phase */
  /// Module indexed by `ModuleIdentifier`.
  pub(crate) modules: rollback::RollbackMap<ModuleIdentifier, BoxModule>,

  /// Dependencies indexed by `DependencyId`.
  dependencies: HashMap<DependencyId, BoxDependency>,
  /// AsyncDependenciesBlocks indexed by `AsyncDependenciesBlockIdentifier`.
  blocks: HashMap<AsyncDependenciesBlockIdentifier, Box<AsyncDependenciesBlock>>,

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
  ///       .unwrap();
  ///     assert_eq!(parents_info.module, parent_module_id);
  ///   })
  /// ```
  dependency_id_to_parents: HashMap<DependencyId, DependencyParents>,
  // TODO try move condition as connection field
  connection_to_condition: HashMap<DependencyId, DependencyCondition>,

  /************************** Modified by Seal Phase **********************/
  /// ModuleGraphModule indexed by `ModuleIdentifier`.
  /// modified here https://github.com/web-infra-dev/rspack/blob/9ae2f0f3be22370197cd9ed3308982f84f2bb738/crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs#L1216
  module_graph_modules: rollback::OverlayMap<ModuleIdentifier, ModuleGraphModule>,

  /// ModuleGraphConnection indexed by `DependencyId`.
  /// modified here https://github.com/web-infra-dev/rspack/blob/9ae2f0f3be22370197cd9ed3308982f84f2bb738/crates/rspack_plugin_javascript/src/plugin/module_concatenation_plugin.rs#L820
  connections: rollback::OverlayMap<DependencyId, ModuleGraphConnection>,

  /***************** only Modified during Seal Phase ********************/
  // setting here https://github.com/web-infra-dev/rspack/blob/9ae2f0f3be22370197cd9ed3308982f84f2bb738/crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs#L318
  dep_meta_map: HashMap<DependencyId, DependencyExtraMeta>,
}
impl ModuleGraphData {
  fn checkpoint(&mut self) {
    self.modules.checkpoint();
    self.module_graph_modules.checkpoint();
    self.connections.checkpoint();
    // dep_meta_map is not used for build_module_graph
  }
  // reset to checkpoint
  fn recover(&mut self) {
    self.modules.reset();
    self.module_graph_modules.reset();
    self.connections.reset();
    // reset data to save memory
    self.dep_meta_map.clear();
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  pub(super) inner: ModuleGraphData,
}
impl ModuleGraph {
  // checkpoint
  pub fn checkpoint(&mut self) {
    self.inner.checkpoint()
  }
  // reset to last checkpoint
  pub fn reset(&mut self) {
    self.inner.recover()
  }
}

impl ModuleGraph {
  #[inline]
  pub fn modules_len(&self) -> usize {
    self.inner.modules.len()
  }

  #[inline]
  pub fn modules(&self) -> impl Iterator<Item = (&ModuleIdentifier, &BoxModule)> {
    self.inner.modules.iter()
  }

  #[inline]
  pub fn modules_par(
    &self,
  ) -> impl rayon::prelude::ParallelIterator<Item = (&ModuleIdentifier, &BoxModule)> {
    self.inner.modules.par_iter()
  }

  #[inline]
  pub fn modules_keys(&self) -> impl Iterator<Item = &ModuleIdentifier> {
    self.inner.modules.iter().map(|(k, _)| k)
  }

  #[inline]
  pub fn module_graph_modules(
    &self,
  ) -> impl Iterator<Item = (&ModuleIdentifier, &ModuleGraphModule)> {
    self.inner.module_graph_modules.iter()
  }

  // #[tracing::instrument(skip_all, fields(module = ?module_id))]
  pub fn get_outcoming_connections_by_module(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashMap<ModuleIdentifier, Vec<&ModuleGraphConnection>> {
    let connections = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have mgm")
      .outgoing_connections();

    let mut map: HashMap<ModuleIdentifier, Vec<&ModuleGraphConnection>> = HashMap::default();
    for dep_id in connections {
      let con = self
        .connection_by_dependency_id(dep_id)
        .expect("should have connection");
      map.entry(*con.module_identifier()).or_default().push(con);
    }
    map
  }

  pub fn get_active_outcoming_connections_by_module(
    &self,
    module_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> HashMap<ModuleIdentifier, Vec<&ModuleGraphConnection>> {
    let connections = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have mgm")
      .outgoing_connections();

    let mut map: HashMap<ModuleIdentifier, Vec<&ModuleGraphConnection>> = HashMap::default();
    for dep_id in connections {
      let con = self
        .connection_by_dependency_id(dep_id)
        .expect("should have connection");
      if !con.is_active(
        module_graph,
        runtime,
        module_graph_cache,
        exports_info_artifact,
      ) {
        continue;
      }
      map.entry(*con.module_identifier()).or_default().push(con);
    }
    map
  }

  // #[tracing::instrument(skip_all, fields(module = ?module_id))]
  pub fn get_incoming_connections_by_origin_module(
    &self,
    module_id: &ModuleIdentifier,
  ) -> HashMap<Option<ModuleIdentifier>, Vec<&ModuleGraphConnection>> {
    let connections = self
      .module_graph_module_by_identifier(module_id)
      .expect("should have mgm")
      .incoming_connections();

    let mut map: HashMap<Option<ModuleIdentifier>, Vec<&ModuleGraphConnection>> =
      HashMap::default();
    for dep_id in connections {
      let con = self
        .connection_by_dependency_id(dep_id)
        .expect("should have connection");
      map
        .entry(con.original_module_identifier)
        .or_default()
        .push(con);
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
    let parent_block = self.get_parent_block(dep_id).copied();

    if module_identifier.is_some() {
      self.inner.connections.remove(dep_id);
    }
    if force {
      self.inner.dependencies.remove(dep_id);
      self.inner.dependency_id_to_parents.remove(dep_id);
      self.inner.connection_to_condition.remove(dep_id);
      if let Some(m_id) = original_module_identifier
        && let Some(module) = self.inner.modules.get_mut(&m_id)
      {
        module.remove_dependency_id(*dep_id);
      }
      if let Some(b_id) = parent_block
        && let Some(block) = self.inner.blocks.get_mut(&b_id)
      {
        block.remove_dependency_id(*dep_id);
      }
    }

    // remove outgoing from original module graph module
    if let Some(original_module_identifier) = &original_module_identifier
      && let Some(mgm) =
        try_get_module_graph_module_mut_by_identifier(self, original_module_identifier)
    {
      mgm.remove_outgoing_connection(dep_id);
      if force {
        mgm.all_dependencies.retain(|id| id != dep_id);
      }
    }
    // remove incoming from module graph module
    if let Some(module_identifier) = &module_identifier
      && let Some(mgm) = try_get_module_graph_module_mut_by_identifier(self, module_identifier)
    {
      mgm.remove_incoming_connection(dep_id);
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

    self.inner.modules.remove(module_id);
    self.inner.module_graph_modules.remove(module_id);

    for block in blocks {
      self.inner.blocks.remove(&block);
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
    self
      .inner
      .module_graph_modules
      .insert(module_graph_module.module_identifier, module_graph_module);
  }

  /// Make sure both source and target module are exists in module graph
  pub fn clone_module_attributes(
    compilation: &mut Compilation,
    source_module: &ModuleIdentifier,
    target_module: &ModuleIdentifier,
  ) {
    let module_graph = compilation.get_module_graph_mut();
    let old_mgm = module_graph
      .module_graph_module_by_identifier(source_module)
      .expect("should have mgm");

    // Using this tuple to avoid violating rustc borrow rules
    let assign_tuple = (
      old_mgm.post_order_index,
      old_mgm.pre_order_index,
      old_mgm.depth,
    );
    let new_mgm = module_graph.module_graph_module_by_identifier_mut(target_module);
    new_mgm.post_order_index = assign_tuple.0;
    new_mgm.pre_order_index = assign_tuple.1;
    new_mgm.depth = assign_tuple.2;

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info(source_module);
    compilation
      .exports_info_artifact
      .set_exports_info(*target_module, exports_info);

    let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, source_module);
    ModuleGraph::set_async(
      &mut compilation.async_modules_artifact,
      *target_module,
      is_async,
    );
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
      let dependency = self.dependency_by_id(&dep_id);
      if filter_connection(connection, dependency) {
        let connection = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        connection.original_module_identifier = Some(*new_module);
        affected_outgoing_connection.push(dep_id);
      }
    }

    let old_mgm = self.module_graph_module_by_identifier_mut(old_module);
    for dep_id in &affected_outgoing_connection {
      old_mgm.remove_outgoing_connection(dep_id);
    }

    let new_mgm = self.module_graph_module_by_identifier_mut(new_module);
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
      let dependency = self.dependency_by_id(&dep_id);
      if filter_connection(connection, dependency) {
        let connection = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        connection.set_module_identifier(*new_module);
        affected_incoming_connection.push(dep_id);
      }
    }

    let old_mgm = self.module_graph_module_by_identifier_mut(old_module);
    for dep_id in &affected_incoming_connection {
      old_mgm.remove_incoming_connection(dep_id);
    }

    let new_mgm = self.module_graph_module_by_identifier_mut(new_module);
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
      let dep = self.dependency_by_id(&dep_id);
      if filter_connection(connection, dep) {
        let con = self
          .connection_by_dependency_id_mut(&dep_id)
          .expect("should have connection");
        con.original_module_identifier = Some(*new_module);
        affected_outgoing_connections.push(dep_id);
      }
    }

    let new_mgm = self.module_graph_module_by_identifier_mut(new_module);
    for dep_id in affected_outgoing_connections {
      new_mgm.add_outgoing_connection(dep_id);
    }
  }

  pub fn get_depth(&self, module_id: &ModuleIdentifier) -> Option<usize> {
    self
      .module_graph_module_by_identifier(module_id)
      .and_then(|mgm| mgm.depth)
  }

  pub fn set_depth_if_lower(&mut self, module_id: &ModuleIdentifier, depth: usize) -> bool {
    let mgm = self.module_graph_module_by_identifier_mut(module_id);
    match mgm.depth {
      Some(cur_depth) if cur_depth <= depth => false,
      _ => {
        mgm.depth = Some(depth);
        true
      }
    }
  }

  pub fn add_module(&mut self, module: BoxModule) {
    self.inner.modules.insert(module.identifier(), module);
  }

  pub fn add_block(&mut self, block: Box<AsyncDependenciesBlock>) {
    self.inner.blocks.insert(block.identifier(), block);
  }

  pub fn set_parents(&mut self, dependency_id: DependencyId, parents: DependencyParents) {
    self
      .inner
      .dependency_id_to_parents
      .insert(dependency_id, parents);
  }

  pub fn get_parent_module(&self, dependency_id: &DependencyId) -> Option<&ModuleIdentifier> {
    self
      .inner
      .dependency_id_to_parents
      .get(dependency_id)
      .map(|p| &p.module)
  }

  pub fn get_parent_block(
    &self,
    dependency_id: &DependencyId,
  ) -> Option<&AsyncDependenciesBlockIdentifier> {
    self
      .inner
      .dependency_id_to_parents
      .get(dependency_id)
      .and_then(|p| p.block.as_ref())
  }

  pub fn get_parent_block_index(&self, dependency_id: &DependencyId) -> Option<usize> {
    self
      .inner
      .dependency_id_to_parents
      .get(dependency_id)
      .map(|p| p.index_in_block)
  }

  pub fn block_by_id(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> Option<&AsyncDependenciesBlock> {
    self.inner.blocks.get(block_id).map(AsRef::as_ref)
  }

  pub fn block_by_id_expect(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
  ) -> &AsyncDependenciesBlock {
    self
      .inner
      .blocks
      .get(block_id)
      .expect("should insert block before get it")
  }

  pub fn blocks(&self) -> &HashMap<AsyncDependenciesBlockIdentifier, Box<AsyncDependenciesBlock>> {
    &self.inner.blocks
  }

  pub fn dependencies(&self) -> impl Iterator<Item = (&DependencyId, &BoxDependency)> {
    self.inner.dependencies.iter()
  }

  pub fn add_dependency(&mut self, dependency: BoxDependency) {
    self.inner.dependencies.insert(*dependency.id(), dependency);
  }

  /// Get a dependency by ID, panicking if not found.
  ///
  /// **PREFERRED METHOD**: Use this for ALL internal Rust code including:
  /// - Core compilation logic
  /// - All plugins (`rspack_plugin_*`)
  /// - Stats generation, code generation, runtime templates
  /// - Chunk graph building, export analysis
  /// - Module graph building operations
  /// - Any internal operations where dependencies should exist
  ///
  /// Dependencies should always be accessible in internal operations, so this
  /// method enforces that invariant with a clear panic message if violated.
  ///
  /// **Only the binding layer (`rspack_binding_api`) should use `internal::try_dependency_by_id()`**
  /// for graceful handling of missing dependencies in external APIs.
  pub fn dependency_by_id(&self, dependency_id: &DependencyId) -> &BoxDependency {
    self
      .inner
      .dependencies
      .get(dependency_id)
      .unwrap_or_else(|| panic!("Dependency with ID {dependency_id:?} not found"))
  }

  /// Get a mutable dependency by ID, panicking if not found.
  ///
  /// **PREFERRED METHOD**: Use this for ALL internal Rust code when you need to
  /// modify dependencies. Dependencies should always be accessible in internal
  /// operations, so this method enforces that invariant with a clear panic message.
  pub fn dependency_by_id_mut(&mut self, dependency_id: &DependencyId) -> &mut BoxDependency {
    self
      .inner
      .dependencies
      .get_mut(dependency_id)
      .unwrap_or_else(|| panic!("Dependency with ID {dependency_id:?} not found"))
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
      .inner
      .connections
      .get(dep_id)
      .map(|con| con.module_identifier())
  }

  pub fn get_module_by_dependency_id(&self, dep_id: &DependencyId) -> Option<&BoxModule> {
    self
      .module_identifier_by_dependency_id(dep_id)
      .and_then(|module_id| self.inner.modules.get(module_id))
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

    if let Some(condition) = condition {
      self
        .inner
        .connection_to_condition
        .insert(connection.dependency_id, condition);
    }

    let module_id = *connection.module_identifier();
    let origin_module_id = connection.original_module_identifier;
    let dependency_id = connection.dependency_id;

    // add to connections list
    self
      .inner
      .connections
      .insert(connection.dependency_id, connection);

    // set to module incoming connection
    {
      let mgm = self.module_graph_module_by_identifier_mut(&module_id);

      mgm.add_incoming_connection(dependency_id);
    }

    // set to origin module outgoing connection
    if let Some(identifier) = origin_module_id {
      let original_mgm = self.module_graph_module_by_identifier_mut(&identifier);
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
    let dependency = self.dependency_by_id(&dependency_id);
    let is_module_dependency =
      dependency.as_module_dependency().is_some() || dependency.as_context_dependency().is_some();
    let condition = dependency
      .as_module_dependency()
      .and_then(|dep| dep.get_condition());
    if !is_module_dependency {
      return Ok(());
    }

    let conditional = condition.is_some();
    let new_connection = ModuleGraphConnection::new(
      dependency_id,
      original_module_identifier,
      module_identifier,
      conditional,
    );
    self.add_connection(new_connection, condition);

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self.inner.modules.get(identifier)
  }

  pub fn module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut BoxModule> {
    self.inner.modules.get_mut(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<&ModuleGraphModule> {
    self.inner.module_graph_modules.get(identifier)
  }

  /// Get a mutable module graph module by identifier, panicking if not found.
  ///
  /// **PREFERRED METHOD**: Use this for all internal code where the module graph module
  /// should exist. This enforces the invariant with a clear panic message if violated.
  ///
  /// Only use `try_module_graph_module_by_identifier_mut()` when you need to handle missing modules.
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> &mut ModuleGraphModule {
    self
      .inner
      .module_graph_modules
      .get_mut(identifier)
      .unwrap_or_else(|| panic!("ModuleGraphModule with identifier {identifier:?} not found"))
  }

  pub fn get_ordered_outgoing_connections(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> impl Iterator<Item = &ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|m| {
        m.all_dependencies
          .iter()
          .filter_map(|dep_id| self.connection_by_dependency_id(dep_id))
      })
      .into_iter()
      .flatten()
  }

  pub fn get_outgoing_deps_in_order(
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
    self.inner.connections.get(dependency_id)
  }

  pub fn get_resolved_module(&self, dependency_id: &DependencyId) -> Option<&ModuleIdentifier> {
    match self.connection_by_dependency_id(dependency_id) {
      Some(connection) => Some(&connection.resolved_module),
      None => None,
    }
  }

  pub fn connection_by_dependency_id_mut(
    &mut self,
    dependency_id: &DependencyId,
  ) -> Option<&mut ModuleGraphConnection> {
    self.inner.connections.get_mut(dependency_id)
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

  pub fn is_optional(
    &self,
    module_id: &ModuleIdentifier,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> bool {
    let mut has_connections = false;
    for connection in self.get_incoming_connections(module_id) {
      let dependency = self.dependency_by_id(&connection.dependency_id);
      let Some(module_dependency) = dependency.as_module_dependency() else {
        return false;
      };
      if !module_dependency.get_optional()
        || !connection.is_target_active(self, None, module_graph_cache, exports_info_artifact)
      {
        return false;
      }
      has_connections = true;
    }
    has_connections
  }

  pub fn is_async(
    async_modules_artifact: &AsyncModulesArtifact,
    module_id: &ModuleIdentifier,
  ) -> bool {
    async_modules_artifact.contains(module_id)
  }

  pub fn is_deferred(
    &self,
    imported_by_defer_modules_artifact: &ImportedByDeferModulesArtifact,
    module_id: &ModuleIdentifier,
  ) -> bool {
    let imported_by_defer = imported_by_defer_modules_artifact.contains(module_id);
    if !imported_by_defer {
      return false;
    }
    let module = self
      .module_by_identifier(module_id)
      .expect("should have module");
    !module.build_meta().has_top_level_await
  }

  pub fn set_async(
    async_modules_artifact: &mut AsyncModulesArtifact,
    module_id: ModuleIdentifier,
    is_async: bool,
  ) -> bool {
    let original = Self::is_async(async_modules_artifact, &module_id);
    if original == is_async {
      return false;
    }
    if original {
      async_modules_artifact.remove(&module_id)
    } else {
      async_modules_artifact.insert(module_id)
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
    self.inner.dep_meta_map.get(id)
  }

  pub fn set_dependency_extra_meta(&mut self, dep_id: DependencyId, extra: DependencyExtraMeta) {
    self.inner.dep_meta_map.insert(dep_id, extra);
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
    let old_mgm = self.module_graph_module_by_identifier_mut(&old_module_identifier);
    old_mgm.remove_incoming_connection(dep_id);

    // add dep_id to updated module mgm incoming connection
    let new_mgm = self.module_graph_module_by_identifier_mut(module_id);
    new_mgm.add_incoming_connection(*dep_id);
  }

  pub fn get_optimization_bailout_mut(&mut self, id: &ModuleIdentifier) -> &mut Vec<String> {
    let mgm = self.module_graph_module_by_identifier_mut(id);
    mgm.optimization_bailout_mut()
  }

  pub fn get_optimization_bailout(&self, id: &ModuleIdentifier) -> &Vec<String> {
    let mgm = self
      .module_graph_module_by_identifier(id)
      .expect("should have module graph module");
    &mgm.optimization_bailout
  }

  pub fn get_condition_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let condition = self
      .inner
      .connection_to_condition
      .get(&connection.dependency_id)
      .expect("should have condition");
    condition.get_connection_state(
      connection,
      runtime,
      self,
      module_graph_cache,
      exports_info_artifact,
    )
  }

  // todo remove it after module_graph_partial remove all of dependency_id_to_*
  pub fn cache_recovery_connection(&mut self, connection: ModuleGraphConnection) {
    let condition = self
      .dependency_by_id(&connection.dependency_id)
      .as_module_dependency()
      .and_then(|dep| dep.get_condition());

    // recovery condition
    if let Some(condition) = condition {
      self
        .inner
        .connection_to_condition
        .insert(connection.dependency_id, condition);
    }

    self
      .inner
      .connections
      .insert(connection.dependency_id, connection);
  }

  pub fn batch_set_connections_original_module(
    &mut self,
    tasks: Vec<(DependencyId, ModuleIdentifier)>,
  ) {
    let changed = tasks
      .into_par_iter()
      .map(|(dep_id, original_module_identifier)| {
        let mut con = self
          .connection_by_dependency_id(&dep_id)
          .expect("should have connection")
          .clone();
        con.original_module_identifier = Some(original_module_identifier);
        (dep_id, con)
      })
      .collect::<Vec<_>>();

    for (dep_id, con) in changed {
      self.inner.connections.insert(dep_id, con);
    }
  }

  pub fn batch_set_connections_module(&mut self, tasks: Vec<(DependencyId, ModuleIdentifier)>) {
    let changed = tasks
      .into_par_iter()
      .map(|(dep_id, module_identifier)| {
        let mut con = self
          .connection_by_dependency_id(&dep_id)
          .expect("should have connection")
          .clone();
        con.set_module_identifier(module_identifier);
        (dep_id, con)
      })
      .collect::<Vec<_>>();

    for (dep_id, con) in changed {
      self.inner.connections.insert(dep_id, con);
    }
  }

  pub fn batch_add_connections(
    &mut self,
    tasks: Vec<(ModuleIdentifier, Vec<DependencyId>, Vec<DependencyId>)>,
  ) {
    let changed = tasks
      .into_par_iter()
      .map(|(mid, outgoings, incomings)| {
        let mut mgm = self
          .module_graph_module_by_identifier(&mid)
          .expect("should have mgm")
          .clone();
        for outgoing in outgoings {
          mgm.add_outgoing_connection(outgoing);
        }
        for incoming in incomings {
          mgm.add_incoming_connection(incoming);
        }
        (mid, mgm)
      })
      .collect::<Vec<_>>();

    for (mid, mgm) in changed {
      self.inner.module_graph_modules.insert(mid, mgm);
    }
  }

  pub fn batch_remove_connections(
    &mut self,
    tasks: Vec<(ModuleIdentifier, Vec<DependencyId>, Vec<DependencyId>)>,
  ) {
    let changed = tasks
      .into_par_iter()
      .map(|(mid, outgoings, incomings)| {
        let mut mgm = self
          .module_graph_module_by_identifier(&mid)
          .expect("should have mgm")
          .clone();
        for outgoing in outgoings.iter() {
          mgm.remove_outgoing_connection(outgoing);
        }
        for incoming in incomings.iter() {
          mgm.remove_incoming_connection(incoming);
        }
        (mid, mgm)
      })
      .collect::<Vec<_>>();

    for (mid, mgm) in changed {
      self.inner.module_graph_modules.insert(mid, mgm);
    }
  }

  pub fn batch_set_export_info_used_name(
    &mut self,
    exports_info_artifact: &mut ExportsInfoArtifact,
    tasks: Vec<(ExportInfo, UsedNameItem)>,
  ) {
    for (export_info, used_name) in tasks {
      let ExportInfo {
        exports_info,
        export_name,
      } = export_info;

      let data = exports_info_artifact.get_exports_info_mut_by_id(&exports_info);
      match export_name {
        ExportName::Named(name) => {
          data
            .named_exports_mut(&name)
            .expect("should have named export")
            .set_used_name(used_name);
        }
        ExportName::Other => {
          data.other_exports_info_mut().set_used_name(used_name);
        }
        ExportName::SideEffects => {
          data.side_effects_only_info_mut().set_used_name(used_name);
        }
      }
    }
  }
}
