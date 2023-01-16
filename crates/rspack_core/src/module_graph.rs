use std::cmp::PartialEq;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use rspack_error::{internal_error, Error, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  BoxModule, BoxModuleDependency, IdentifierMap, Module, ModuleGraphModule, ModuleIdentifier,
};

// FIXME: placing this as global id is not acceptable, move it to somewhere else later
static NEXT_MODULE_GRAPH_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,
  /// The referencing dependency id
  pub dependency_id: usize,

  /// The unique id of this connection
  pub id: usize,
}

impl Hash for ModuleGraphConnection {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.original_module_identifier.hash(state);
    self.module_identifier.hash(state);
    self.dependency_id.hash(state);
  }
}

impl PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.original_module_identifier == other.original_module_identifier
      && self.module_identifier == other.module_identifier
      && self.dependency_id == other.dependency_id
  }
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: usize,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency_id,

      id: NEXT_MODULE_GRAPH_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  dependency_id_to_module_identifier: HashMap<usize, ModuleIdentifier>,

  /// Module identifier to its module
  pub(crate) module_identifier_to_module: IdentifierMap<BoxModule>,
  /// Module identifier to its module graph module
  pub(crate) module_identifier_to_module_graph_module: IdentifierMap<ModuleGraphModule>,

  dependency_id_to_connection_id: HashMap<usize, usize>,
  connection_id_to_dependency_id: HashMap<usize, usize>,
  dependency_id_to_dependency: HashMap<usize, BoxModuleDependency>,
  dependency_to_dependency_id: HashMap<BoxModuleDependency, usize>,

  /// The module graph connections
  connections: HashSet<ModuleGraphConnection>,
  connection_id_to_connection: HashMap<usize, ModuleGraphConnection>,
}

impl ModuleGraph {
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

  pub fn add_dependency(
    &mut self,
    dep: BoxModuleDependency,
    module_identifier: ModuleIdentifier,
  ) -> usize {
    static NEXT_DEPENDENCY_ID: AtomicUsize = AtomicUsize::new(0);

    let id = NEXT_DEPENDENCY_ID.fetch_add(1, Ordering::Relaxed);
    self.dependency_id_to_dependency.insert(id, dep.clone());
    self.dependency_to_dependency_id.insert(dep, id);

    self
      .dependency_id_to_module_identifier
      .insert(id, module_identifier);

    id
  }

  /// Uniquely identify a module by its dependency
  pub fn module_by_dependency(&self, dep: &BoxModuleDependency) -> Option<&ModuleGraphModule> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_module_identifier.get(id))
      .and_then(|module_identifier| {
        self
          .module_identifier_to_module_graph_module
          .get(module_identifier)
      })
  }

  /// Get the dependency id of a dependency
  pub fn dependency_id_by_dependency(&self, dep: &BoxModuleDependency) -> Option<usize> {
    self.dependency_to_dependency_id.get(dep).cloned()
  }

  /// Return an unordered iterator of module graph modules
  pub fn module_graph_modules(&self) -> impl Iterator<Item = &ModuleGraphModule> {
    self.module_identifier_to_module_graph_module.values()
  }

  /// Return an unordered iterator of modules
  pub fn modules(&self) -> impl Iterator<Item = &BoxModule> {
    self.module_identifier_to_module.values()
  }

  /// Add a connection between two module graph modules, if a connection exists, then it will be reused.
  pub fn set_resolved_module(
    &mut self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: usize,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let new_connection =
      ModuleGraphConnection::new(original_module_identifier, dependency_id, module_identifier);

    let connection_id = if let Some(connection) = self.connections.get(&new_connection) {
      connection.id
    } else {
      let id = new_connection.id;
      self.connections.insert(new_connection.clone());
      self.connection_id_to_connection.insert(id, new_connection);
      id
    };

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
          Error::InternalError(internal_error!(format!(
            "Failed to set resolved module: Module linked to module identifier {module_identifier} cannot be found"
          )))
        })?;

      mgm.add_incoming_connection(connection_id);
    }

    if let Some(identifier) = original_module_identifier && let Some(original_mgm) = self.
    module_graph_module_by_identifier_mut(&identifier) {
        original_mgm.add_outgoing_connection(connection_id);
    };

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  #[inline]
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self.module_identifier_to_module.get(identifier)
  }

  /// Uniquely identify a module by its identifier and return the exclusive reference
  #[inline]
  pub fn module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut BoxModule> {
    self.module_identifier_to_module.get_mut(identifier)
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

  /// Uniquely identify a connection by a given dependency
  pub fn connection_by_dependency(
    &self,
    dep: &BoxModuleDependency,
  ) -> Option<&ModuleGraphConnection> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_connection_id.get(id))
      .and_then(|id| self.connection_id_to_connection.get(id))
  }

  /// Get a list of all dependencies of a module by the module itself, if the module is not found, then None is returned
  pub fn dependencies_by_module(&self, module: &dyn Module) -> Option<&[BoxModuleDependency]> {
    self.dependencies_by_module_identifier(&module.identifier())
  }

  /// Get a list of all dependencies of a module by the module identifier, if the module is not found, then None is returned
  pub fn dependencies_by_module_identifier(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&[BoxModuleDependency]> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(|mgm| mgm.dependencies.as_slice())
  }

  pub fn dependency_by_connection(
    &self,
    connection: &ModuleGraphConnection,
  ) -> Option<&BoxModuleDependency> {
    self.dependency_by_connection_id(connection.id)
  }

  pub fn dependency_by_connection_id(&self, connection_id: usize) -> Option<&BoxModuleDependency> {
    self
      .connection_id_to_dependency_id
      .get(&connection_id)
      .and_then(|id| self.dependency_id_to_dependency.get(id))
  }

  pub fn connection_by_connection_id(
    &self,
    connection_id: usize,
  ) -> Option<&ModuleGraphConnection> {
    self.connection_id_to_connection.get(&connection_id)
  }

  pub fn remove_connection_by_dependency(&mut self, dep: &BoxModuleDependency) {
    if let Some(id) = self.dependency_to_dependency_id.get(dep).copied() {
      if let Some(conn) = self.dependency_id_to_connection_id.remove(&id) {
        self.connection_id_to_dependency_id.remove(&conn);

        if let Some(conn) = self.connection_id_to_connection.remove(&conn) {
          self.connections.remove(&conn);

          if let Some(mgm) = conn
            .original_module_identifier
            .as_ref()
            .and_then(|ident| self.module_graph_module_by_identifier_mut(ident))
          {
            mgm.outgoing_connections.remove(&conn.id);
          };

          if let Some(mgm) = self.module_graph_module_by_identifier_mut(&conn.module_identifier) {
            mgm.incoming_connections.remove(&conn.id);
          }
        }
      }
      self.dependency_id_to_module_identifier.remove(&id);
      self.dependency_id_to_dependency.remove(&id);
      self.dependency_to_dependency_id.remove(dep);
    }
  }

  pub fn get_pre_order_index(&self, module_identifier: &ModuleIdentifier) -> Option<usize> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.pre_order_index)
  }

  pub fn get_issuer(&self, module: &BoxModule) -> Option<&BoxModule> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .and_then(|mgm| mgm.get_issuer().get_module(self))
  }

  pub fn get_outgoing_connections(&self, module: &BoxModule) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .map(|mgm| {
        mgm
          .outgoing_connections
          .iter()
          .filter_map(|id| self.connection_by_connection_id(*id))
          .collect()
      })
      .unwrap_or_default()
  }
}

#[cfg(test)]
mod test {
  use std::borrow::Cow;

  use rspack_error::{Result, TWithDiagnosticArray};
  use rspack_sources::Source;

  use crate::{
    BuildContext, BuildResult, CodeGeneratable, CodeGenerationResult, Compilation, Context,
    Dependency, Identifiable, Module, ModuleDependency, ModuleGraph, ModuleGraphModule,
    ModuleIdentifier, ModuleType, SourceType,
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

        fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
          unreachable!()
        }
      }
    };
  }

  impl_noop_trait_module_type!(Node);

  // Define a detailed edge type for `ModuleGraphConnection`s, tuple contains the parent module identifier and the child module specifier
  #[derive(Debug, PartialEq, Eq, Hash, Clone)]
  struct Edge(Option<ModuleIdentifier>, String);

  macro_rules! impl_noop_trait_dep_type {
    ($ident:ident) => {
      impl Dependency for $ident {
        fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
          self.0.as_ref()
        }
      }

      impl ModuleDependency for $ident {
        fn request(&self) -> &str {
          &*self.1
        }

        fn user_request(&self) -> &str {
          &*self.1
        }

        fn span(&self) -> Option<&crate::ErrorSpan> {
          unreachable!()
        }
      }

      impl CodeGeneratable for $ident {
        fn generate(
          &self,
          _code_generatable_context: &mut crate::CodeGeneratableContext,
        ) -> Result<crate::CodeGeneratableResult> {
          unreachable!()
        }
      }
    };
  }

  impl_noop_trait_dep_type!(Edge);

  fn add_module_to_graph(mg: &mut ModuleGraph, m: Box<dyn Module>) {
    let mgm = ModuleGraphModule::new(None, m.identifier(), ModuleType::Js, true);
    mg.add_module_graph_module(mgm);
    mg.add_module(m);
  }

  fn link_modules_with_dependency(
    mg: &mut ModuleGraph,
    from: Option<&ModuleIdentifier>,
    to: &ModuleIdentifier,
    dep: Box<dyn ModuleDependency>,
  ) {
    let did = mg.add_dependency(dep.clone(), *to);
    if let Some(p_id) = from && let Some(mgm) = mg.module_graph_module_by_identifier_mut(p_id) {
      mgm.dependencies.push(dep);
    }
    mg.set_resolved_module(from.copied(), did, *to)
      .expect("failed to set resolved module");

    assert_eq!(
      mg.dependency_id_to_module_identifier.get(&did).copied(),
      Some(*to)
    );
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
      Edge(Some($from.into()), $to.into())
    };
    ($from:expr, $to:expr) => {
      Edge($from, $to.into())
    };
  }

  #[test]
  fn test_module_graph() {
    let mut mg = ModuleGraph::default();
    let a = node!("a");
    let b = node!("b");
    let a_id = a.identifier();
    let b_id = b.identifier();
    let a_to_b = edge!(Some(a_id.clone()), b_id.as_str());
    add_module_to_graph(&mut mg, box a);
    add_module_to_graph(&mut mg, box b);
    link_modules_with_dependency(&mut mg, Some(&a_id), &b_id, box (a_to_b.clone()));

    let mgm_a = mgm(&mg, &a_id);
    let mgm_b = mgm(&mg, &b_id);
    let conn_a = mgm_a.outgoing_connections.iter().collect::<Vec<_>>();
    let conn_b = mgm_b.incoming_connections.iter().collect::<Vec<_>>();
    assert_eq!(conn_a[0], conn_b[0]);

    let c = node!("c");
    let c_id = c.identifier();
    let b_to_c = edge!(Some(b_id.clone()), c_id.as_str());
    add_module_to_graph(&mut mg, box c);
    link_modules_with_dependency(&mut mg, Some(&b_id), &c_id, box (b_to_c.clone()));

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    let conn_b = mgm_b.outgoing_connections.iter().collect::<Vec<_>>();
    let conn_c = mgm_c.incoming_connections.iter().collect::<Vec<_>>();
    assert_eq!(conn_c[0], conn_b[0]);

    mg.remove_connection_by_dependency(&(box a_to_b as Box<dyn ModuleDependency>));

    let mgm_a = mgm(&mg, &a_id);
    let mgm_b = mgm(&mg, &b_id);
    assert!(mgm_a.outgoing_connections.is_empty());
    assert!(mgm_b.incoming_connections.is_empty());

    mg.remove_connection_by_dependency(&(box b_to_c as Box<dyn ModuleDependency>));

    let mgm_b = mgm(&mg, &b_id);
    let mgm_c = mgm(&mg, &c_id);
    assert!(mgm_b.outgoing_connections.is_empty());
    assert!(mgm_c.incoming_connections.is_empty());
  }
}
