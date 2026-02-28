use rspack_cacheable::{cacheable, with::Skip};
use rustc_hash::FxHashSet as HashSet;

use crate::{DependencyId, ModuleIdentifier, ModuleIssuer};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleGraphModule {
  // edges from module to module
  outgoing_connections: HashSet<DependencyId>,
  // incoming connections will regenerate by persistent cache recovery.
  #[cacheable(with=Skip)]
  incoming_connections: HashSet<DependencyId>,

  issuer: ModuleIssuer,

  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // an quick way to get a module's all dependencies (including its blocks' dependencies)
  // and it is ordered by dependency creation order
  pub(crate) all_dependencies: Vec<DependencyId>,
  pub(crate) pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub depth: Option<usize>,
  pub optimization_bailout: Vec<String>,
}

impl ModuleGraphModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    Self {
      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),
      issuer: ModuleIssuer::Unset,
      // exec_order: usize::MAX,
      module_identifier,
      all_dependencies: Default::default(),
      pre_order_index: None,
      post_order_index: None,
      depth: None,
      optimization_bailout: vec![],
    }
  }

  pub fn add_incoming_connection(&mut self, dependency_id: DependencyId) {
    self.incoming_connections.insert(dependency_id);
  }

  pub fn remove_incoming_connection(&mut self, dependency_id: &DependencyId) {
    self.incoming_connections.remove(dependency_id);
  }

  pub fn add_outgoing_connection(&mut self, dependency_id: DependencyId) {
    self.outgoing_connections.insert(dependency_id);
  }

  pub fn remove_outgoing_connection(&mut self, dependency_id: &DependencyId) {
    self.outgoing_connections.remove(dependency_id);
  }

  pub fn incoming_connections(&self) -> &HashSet<DependencyId> {
    &self.incoming_connections
  }

  pub fn outgoing_connections(&self) -> &HashSet<DependencyId> {
    &self.outgoing_connections
  }

  pub fn set_issuer_if_unset(&mut self, issuer: Option<ModuleIdentifier>) {
    if matches!(self.issuer, ModuleIssuer::Unset) {
      self.issuer = ModuleIssuer::from_identifier(issuer);
    }
  }

  pub fn set_issuer(&mut self, issuer: ModuleIssuer) {
    self.issuer = issuer;
  }

  pub fn issuer(&self) -> &ModuleIssuer {
    &self.issuer
  }

  pub(crate) fn optimization_bailout_mut(&mut self) -> &mut Vec<String> {
    &mut self.optimization_bailout
  }
}
