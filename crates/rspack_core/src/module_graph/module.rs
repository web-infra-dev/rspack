use rustc_hash::FxHashSet as HashSet;

use crate::ExportsInfoId;
use crate::{
  module_graph::ConnectionId, ChunkGraph, DependencyId, FactoryMeta, ModuleIdentifier,
  ModuleIssuer, ModuleProfile, ModuleSyntax, ModuleType,
};

#[derive(Debug)]
pub struct ModuleGraphModule {
  // edges from module to module
  outgoing_connections: HashSet<ConnectionId>,
  incoming_connections: HashSet<ConnectionId>,

  issuer: ModuleIssuer,

  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  // TODO: remove this once we drop old treeshaking
  pub(crate) __deprecated_all_dependencies: Vec<DependencyId>,
  pub(crate) pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub module_syntax: ModuleSyntax,
  pub factory_meta: Option<FactoryMeta>,
  pub exports: ExportsInfoId,
  pub profile: Option<Box<ModuleProfile>>,
  pub is_async: bool,
  pub depth: Option<usize>,
  pub optimization_bailout: Vec<String>,
}

impl ModuleGraphModule {
  pub fn new(
    module_identifier: ModuleIdentifier,
    module_type: ModuleType,
    exports_info_id: ExportsInfoId,
  ) -> Self {
    Self {
      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),
      issuer: ModuleIssuer::Unset,
      // exec_order: usize::MAX,
      module_identifier,
      __deprecated_all_dependencies: Default::default(),
      module_type,
      pre_order_index: None,
      post_order_index: None,
      module_syntax: ModuleSyntax::empty(),
      factory_meta: None,
      exports: exports_info_id,
      profile: None,
      is_async: false,
      depth: None,
      optimization_bailout: vec![],
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    let c = chunk_graph.get_module_id(self.module_identifier).as_ref();
    c.unwrap_or_else(|| panic!("{} module id not found", self.module_identifier))
      .as_str()
  }

  pub fn add_incoming_connection(&mut self, connection_id: ConnectionId) {
    self.incoming_connections.insert(connection_id);
  }

  pub fn remove_incoming_connection(&mut self, connection_id: &ConnectionId) {
    self.incoming_connections.remove(connection_id);
  }

  pub fn add_outgoing_connection(&mut self, connection_id: ConnectionId) {
    self.outgoing_connections.insert(connection_id);
  }

  pub fn remove_outgoing_connection(&mut self, connection_id: &ConnectionId) {
    self.outgoing_connections.remove(connection_id);
  }

  pub fn incoming_connections(&self) -> &HashSet<ConnectionId> {
    &self.incoming_connections
  }

  pub fn outgoing_connections(&self) -> &HashSet<ConnectionId> {
    &self.outgoing_connections
  }

  pub fn get_incoming_connections_unordered(&self) -> &HashSet<ConnectionId> {
    &self.incoming_connections
  }

  pub fn get_outgoing_connections_unordered(&self) -> &HashSet<ConnectionId> {
    &self.outgoing_connections
  }

  pub fn set_profile(&mut self, profile: Box<ModuleProfile>) {
    self.profile = Some(profile);
  }

  pub fn get_profile(&self) -> Option<&ModuleProfile> {
    self.profile.as_deref()
  }

  pub fn set_issuer_if_unset(&mut self, issuer: Option<ModuleIdentifier>) {
    if matches!(self.issuer, ModuleIssuer::Unset) {
      self.issuer = ModuleIssuer::from_identifier(issuer);
    }
  }

  pub fn set_issuer(&mut self, issuer: ModuleIssuer) {
    self.issuer = issuer;
  }

  pub fn get_issuer(&self) -> &ModuleIssuer {
    &self.issuer
  }

  pub(crate) fn optimization_bailout_mut(&mut self) -> &mut Vec<String> {
    &mut self.optimization_bailout
  }

  pub(crate) fn optimization_bailout(&self) -> &Vec<String> {
    &self.optimization_bailout
  }
}
