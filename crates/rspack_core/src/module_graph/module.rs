use std::fmt;

use rspack_cacheable::{cacheable, with::Skip};
use rustc_hash::FxHashSet;

use crate::{
  DependencyId, DependencyLocation, ModuleGraphConnectionId, ModuleIdentifier, ModuleIssuer,
};

#[cacheable]
#[derive(Debug, Clone)]
pub enum OptimizationBailoutItem {
  Message(String),
  SideEffects {
    node_type: String,
    loc: DependencyLocation,
    short_id: String,
  },
}

impl fmt::Display for OptimizationBailoutItem {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Message(msg) => write!(f, "{msg}"),
      Self::SideEffects {
        node_type,
        loc,
        short_id,
      } => {
        write!(
          f,
          "{node_type} with side_effects in source code at {short_id}:{loc}"
        )
      }
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleGraphModule {
  // edges from module to module
  outgoing_connections: FxHashSet<ModuleGraphConnectionId>,
  // incoming connections will regenerate by persistent cache recovery.
  #[cacheable(with=Skip)]
  incoming_connections: FxHashSet<ModuleGraphConnectionId>,

  issuer: ModuleIssuer,
  // The owner dependency of the factory group that produced this build, `None` for an entry, whose
  // dependency is new to each compiler. `fix_issuers` rewrites the issuer after a forced rebuild,
  // so only this field tells which group decided the build. Persistent cache stores it beside the
  // issuer, outside the module build.
  #[cacheable(with=Skip)]
  producer: Option<DependencyId>,

  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // an quick way to get a module's all dependencies (including its blocks' dependencies)
  // and it is ordered by dependency creation order
  all_dependencies: Vec<DependencyId>,
  pub(crate) pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub depth: Option<usize>,
  pub optimization_bailout: Vec<OptimizationBailoutItem>,
}

impl ModuleGraphModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    Self {
      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),
      issuer: ModuleIssuer::Unset,
      producer: None,
      // exec_order: usize::MAX,
      module_identifier,
      all_dependencies: Default::default(),
      pre_order_index: None,
      post_order_index: None,
      depth: None,
      optimization_bailout: vec![],
    }
  }

  pub fn add_incoming_connection(&mut self, connection_id: ModuleGraphConnectionId) {
    self.incoming_connections.insert(connection_id);
  }

  pub fn remove_incoming_connection(&mut self, connection_id: &ModuleGraphConnectionId) {
    self.incoming_connections.remove(connection_id);
  }

  pub fn add_outgoing_connection(&mut self, connection_id: ModuleGraphConnectionId) {
    self.outgoing_connections.insert(connection_id);
  }

  pub fn remove_outgoing_connection(&mut self, connection_id: &ModuleGraphConnectionId) {
    self.outgoing_connections.remove(connection_id);
  }

  pub fn incoming_connections(&self) -> &FxHashSet<ModuleGraphConnectionId> {
    &self.incoming_connections
  }

  pub fn outgoing_connections(&self) -> &FxHashSet<ModuleGraphConnectionId> {
    &self.outgoing_connections
  }

  pub fn all_dependencies(&self) -> &[DependencyId] {
    &self.all_dependencies
  }

  pub(crate) fn all_dependencies_mut(&mut self) -> &mut Vec<DependencyId> {
    &mut self.all_dependencies
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

  pub(crate) fn set_producer(&mut self, producer: Option<DependencyId>) {
    self.producer = producer;
  }

  pub(crate) fn producer(&self) -> Option<DependencyId> {
    self.producer
  }

  pub(crate) fn optimization_bailout_mut(&mut self) -> &mut Vec<OptimizationBailoutItem> {
    &mut self.optimization_bailout
  }
}
