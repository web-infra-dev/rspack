use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::path::PathBuf;

use rspack_error::{internal_error, Result};
use rspack_hash::RspackHashDigest;
use rspack_identifier::IdentifierMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::JsWord;

use crate::IS_NEW_TREESHAKING;
mod connection;
pub use connection::*;

use crate::{
  to_identifier, BoxDependency, BoxModule, BuildDependency, BuildInfo, BuildMeta,
  DependencyCondition, DependencyId, ExportInfo, ExportInfoId, ExportsInfo, ExportsInfoId, Module,
  ModuleGraphModule, ModuleIdentifier, ModuleProfile,
};

// TODO Here request can be used JsWord
pub type ImportVarMap = HashMap<String /* request */, String /* import_var */>;

#[derive(Debug, Default)]
pub struct ModuleGraph {
  dependency_id_to_module_identifier: HashMap<DependencyId, ModuleIdentifier>,

  /// Module identifier to its module
  pub module_identifier_to_module: IdentifierMap<BoxModule>,

  /// Module identifier to its module graph module
  pub module_identifier_to_module_graph_module: IdentifierMap<ModuleGraphModule>,

  dependency_id_to_connection_id: HashMap<DependencyId, ConnectionId>,
  connection_id_to_dependency_id: HashMap<ConnectionId, DependencyId>,

  /// Dependencies indexed by `DependencyId`
  /// None means the dependency has been removed
  dependencies: HashMap<DependencyId, BoxDependency>,

  /// Dependencies indexed by `ConnectionId`
  /// None means the connection has been removed
  connections: Vec<Option<ModuleGraphConnection>>,

  /// Module graph connections table index for `ConnectionId`
  connections_map: HashMap<ModuleGraphConnection, ConnectionId>,

  import_var_map: IdentifierMap<ImportVarMap>,
  pub exports_info_map: HashMap<ExportsInfoId, ExportsInfo>,
  pub export_info_map: HashMap<ExportInfoId, ExportInfo>,
  connection_to_condition: HashMap<ModuleGraphConnection, DependencyCondition>,
  pub dep_meta_map: HashMap<DependencyId, DependencyExtraMeta>,
}

/// https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ModuleGraph.js#L742-L748
#[derive(Debug)]
pub struct DependencyExtraMeta {
  pub ids: Vec<JsWord>,
}

impl ModuleGraph {
  /// Return an unordered iterator of modules
  pub fn modules(&self) -> &IdentifierMap<BoxModule> {
    &self.module_identifier_to_module
  }

  pub fn modules_mut(&mut self) -> &mut IdentifierMap<BoxModule> {
    &mut self.module_identifier_to_module
  }

  pub fn module_graph_modules(&self) -> &IdentifierMap<ModuleGraphModule> {
    &self.module_identifier_to_module_graph_module
  }

  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    if let Entry::Vacant(val) = self
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier)
    {
      val.insert(module_graph_module);
    }
  }

  pub fn add_module(&mut self, module: BoxModule) {
    if let Entry::Vacant(val) = self.module_identifier_to_module.entry(module.identifier()) {
      val.insert(module);
    }
  }

  pub fn add_dependency(&mut self, dependency: BoxDependency) {
    self.dependencies.insert(*dependency.id(), dependency);
  }

  pub fn dependency_by_id(&self, dependency_id: &DependencyId) -> Option<&BoxDependency> {
    self.dependencies.get(dependency_id)
  }

  pub fn dependency_by_id_mut(
    &mut self,
    dependency_id: &DependencyId,
  ) -> Option<&mut BoxDependency> {
    self.dependencies.get_mut(dependency_id)
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

  /// Add a connection between two module graph modules, if a connection exists, then it will be reused.
  pub fn set_resolved_module(
    &mut self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency: BoxDependency,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let is_module_dependency = dependency.as_module_dependency().is_some();
    let dependency_id = *dependency.id();
    let condition = if IS_NEW_TREESHAKING.load(std::sync::atomic::Ordering::Relaxed) {
      dependency
        .as_module_dependency()
        .and_then(|dep| dep.get_condition())
    } else {
      None
    };
    self.add_dependency(dependency);
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
        .ok_or_else(|| {
          internal_error!(
            "Failed to set resolved module: Module linked to module identifier {module_identifier} cannot be found"
          )
        })?;

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
    export_name: &JsWord,
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

  /// Get a list of all dependencies of a module by the module itself, if the module is not found, then None is returned
  pub fn dependencies_by_module(&self, module: &dyn Module) -> Option<&[DependencyId]> {
    self.dependencies_by_module_identifier(&module.identifier())
  }

  /// Get a list of all dependencies of a module by the module identifier, if the module is not found, then None is returned
  pub fn dependencies_by_module_identifier(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&[DependencyId]> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|mgm| mgm.dependencies.as_slice())
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
          mgm.outgoing_connections.remove(&connection_id);
        };

        if let Some(mgm) = self.module_graph_module_by_identifier_mut(&connection.module_identifier)
        {
          mgm.incoming_connections.remove(&connection_id);
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
          .outgoing_connections
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
          .incoming_connections
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
          .incoming_connections
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
      ..
    } = connection;

    // remove dependency
    self.dependency_id_to_connection_id.remove(&dependency_id);
    self
      .dependency_id_to_module_identifier
      .remove(&dependency_id);

    // remove outgoing from original module graph module
    if let Some(original_module_identifier) = &original_module_identifier {
      if let Some(mgm) = self
        .module_identifier_to_module_graph_module
        .get_mut(original_module_identifier)
      {
        mgm.outgoing_connections.remove(&connection_id);
        // Because of mgm.dependencies is set when original module build success
        // it does not need to remove dependency in mgm.dependencies.
      }
    }
    // remove incoming from module graph module
    if let Some(mgm) = self
      .module_identifier_to_module_graph_module
      .get_mut(&module_identifier)
    {
      mgm.incoming_connections.remove(&connection_id);
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
      for cid in mgm.outgoing_connections {
        self.revoke_connection(cid);
      }

      mgm
        .incoming_connections
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
    if let Some(mgm) = self.module_graph_module_by_identifier_mut(module_identifier) {
      mgm.build_info = Some(build_info);
      mgm.build_meta = Some(build_meta);
    }
  }

  #[inline]
  pub fn get_module_hash(&self, module_identifier: &ModuleIdentifier) -> Option<&RspackHashDigest> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.build_info.as_ref().and_then(|i| i.hash.as_ref()))
  }

  pub fn has_dependencies(
    &self,
    module_identifier: &ModuleIdentifier,
    files: &HashSet<PathBuf>,
  ) -> bool {
    if let Some(build_info) = self
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.build_info.as_ref())
    {
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
    let connection = self
      .connection_by_dependency_mut(dep_id)
      .expect("should have connection");
    if &connection.module_identifier == module_id {
      return;
    }
    connection.set_active(false);
    let mut new_connection = *connection;
    let condition = self.connection_to_condition.get(&new_connection).cloned();
    new_connection.module_identifier = *module_id;
    let new_connection = normalize_new_connection(self, new_connection);

    if let Some(condition) = condition {
      self
        .connection_to_condition
        .insert(new_connection, condition);
    }

    pub fn normalize_new_connection(
      mg: &mut ModuleGraph,
      new_connection: ModuleGraphConnection,
    ) -> ModuleGraphConnection {
      let dependency_id = new_connection.dependency_id;
      let connection_id = if let Some(connection_id) = mg.connections_map.get(&new_connection) {
        *connection_id
      } else {
        let new_connection_id = ConnectionId::from(mg.connections.len());
        mg.connections.push(Some(new_connection));
        mg.connections_map.insert(new_connection, new_connection_id);
        new_connection_id
      };

      mg.dependency_id_to_connection_id
        .insert(dependency_id, connection_id);

      mg.connection_id_to_dependency_id
        .insert(connection_id, dependency_id);

      let mgm = mg
        .module_graph_module_by_identifier_mut(&new_connection.module_identifier)
        .expect("should have mgm");

      mgm.add_incoming_connection(connection_id);

      if let Some(identifier) = new_connection.original_module_identifier
        && let Some(original_mgm) = mg.module_graph_module_by_identifier_mut(&identifier)
      {
        original_mgm.add_outgoing_connection(connection_id);
      };
      new_connection
    }
  }

  pub fn set_dependency_import_var(&mut self, module_identifier: ModuleIdentifier, request: &str) {
    self.import_var_map.entry(module_identifier).or_default();
    if let Some(module_var_map) = self.import_var_map.get_mut(&module_identifier) {
      if !module_var_map.contains_key(request) {
        module_var_map.insert(
          request.to_string(),
          format!(
            "{}__WEBPACK_IMPORTED_MODULE_{}__",
            to_identifier(request),
            module_var_map.len()
          ),
        );
      }
    }
  }

  pub fn get_import_var(&self, module_identifier: &ModuleIdentifier, request: &str) -> &str {
    self
      .import_var_map
      .get(module_identifier)
      .expect("should have module import var")
      .get(request)
      .unwrap_or_else(|| panic!("should have import var for {module_identifier} {request}"))
  }

  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> &ExportsInfo {
    let mgm = self
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    let exports_info = self
      .exports_info_map
      .get(&mgm.exports)
      .expect("should have export info");
    exports_info
  }

  // pub fn get_exports_info_mut(&mut self, module_identifier: &ModuleIdentifier) -> &mut ExportsInfo {
  //   let mgm = self
  //     .module_graph_module_by_identifier_mut(module_identifier)
  //     .expect("should have mgm");
  //   &mut mgm.exports
  // }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfoId) -> &ExportsInfo {
    let exports_info = self
      .exports_info_map
      .get(id)
      .expect("should have exports_info");
    exports_info
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfoId) -> &mut ExportsInfo {
    let exports_info = self
      .exports_info_map
      .get_mut(id)
      .expect("should have exports_info");
    exports_info
  }

  pub fn get_export_info_by_id(&self, id: &ExportInfoId) -> &ExportInfo {
    let export_info = self
      .export_info_map
      .get(id)
      .expect("should have export info");
    export_info
  }

  pub fn get_export_info_mut_by_id(&mut self, id: &ExportInfoId) -> &mut ExportInfo {
    let exports_info = self
      .export_info_map
      .get_mut(id)
      .expect("should have export info");
    exports_info
  }
}

#[cfg(test)]
mod test {
  use std::borrow::Cow;

  use rspack_error::{Result, TWithDiagnosticArray};
  use rspack_identifier::Identifiable;
  use rspack_sources::Source;

  use crate::{
    BoxDependency, BuildContext, BuildResult, CodeGenerationResult, Compilation, Context,
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

        async fn build(
          &mut self,
          _build_context: BuildContext<'_>,
        ) -> Result<TWithDiagnosticArray<BuildResult>> {
          unreachable!()
        }

        fn code_generation(
          &self,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
        ) -> Result<CodeGenerationResult> {
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
      .insert(other_exports_info.id, other_exports_info);
    mg.export_info_map
      .insert(side_effects_only_info.id, side_effects_only_info);
    mg.exports_info_map.insert(exports_info.id, exports_info);
  }

  fn link_modules_with_dependency(
    mg: &mut ModuleGraph,
    from: Option<&ModuleIdentifier>,
    to: &ModuleIdentifier,
    dep: BoxDependency,
  ) -> DependencyId {
    let dependency_id = *dep.id();
    mg.dependency_id_to_module_identifier
      .insert(dependency_id, *to);
    if let Some(p_id) = from
      && let Some(mgm) = mg.module_graph_module_by_identifier_mut(p_id)
    {
      mgm.dependencies.push(dependency_id);
    }
    mg.set_resolved_module(from.copied(), dep, *to)
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
    let conn_a = mgm_a.outgoing_connections.iter().collect::<Vec<_>>();
    let conn_b = mgm_b.incoming_connections.iter().collect::<Vec<_>>();
    assert_eq!(conn_a[0], conn_b[0]);

    let c = node!("c");
    let c_id = c.identifier();
    let b_to_c = edge!(Some(b_id), c_id.as_str());
    add_module_to_graph(&mut mg, Box::new(c));
    let b_to_c_id = link_modules_with_dependency(&mut mg, Some(&b_id), &c_id, Box::new(b_to_c));

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    let conn_b = mgm_b.outgoing_connections.iter().collect::<Vec<_>>();
    let conn_c = mgm_c.incoming_connections.iter().collect::<Vec<_>>();
    assert_eq!(conn_c[0], conn_b[0]);

    mg.remove_connection_by_dependency(&a_to_b_id);

    let mgm_a = mgm(&mg, &a_id);
    let mgm_b = mgm(&mg, &b_id);
    assert!(mgm_a.outgoing_connections.is_empty());
    assert!(mgm_b.incoming_connections.is_empty());

    mg.remove_connection_by_dependency(&b_to_c_id);

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    assert!(mgm_b.outgoing_connections.is_empty());
    assert!(mgm_c.incoming_connections.is_empty());
  }
}
