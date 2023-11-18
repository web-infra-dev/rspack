use rspack_error::{internal_error, Result};
use rustc_hash::FxHashSet as HashSet;

use crate::{
  module_graph::ConnectionId, BuildInfo, BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType,
  ChunkGraph, DependencyId, ExportsArgument, ExportsType, FactoryMeta, ModuleArgument, ModuleGraph,
  ModuleGraphConnection, ModuleIdentifier, ModuleIssuer, ModuleProfile, ModuleSyntax, ModuleType,
};
use crate::{ExportsInfoId, GroupOptions, IS_NEW_TREESHAKING};

#[derive(Debug)]
pub struct ModuleGraphModule {
  // edges from module to module
  pub outgoing_connections: HashSet<ConnectionId>,
  pub incoming_connections: HashSet<ConnectionId>,

  issuer: ModuleIssuer,

  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  // TODO: remove this once we drop old treeshaking
  pub all_dependencies: Box<Vec<DependencyId>>,
  pub(crate) pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub module_syntax: ModuleSyntax,
  pub factory_meta: Option<FactoryMeta>,
  pub build_info: Option<BuildInfo>,
  pub build_meta: Option<BuildMeta>,
  pub exports: ExportsInfoId,
  pub profile: Option<Box<ModuleProfile>>,
  pub is_async: bool,
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
      all_dependencies: Default::default(),
      module_type,
      pre_order_index: None,
      post_order_index: None,
      module_syntax: ModuleSyntax::empty(),
      factory_meta: None,
      build_info: None,
      build_meta: None,
      exports: exports_info_id,
      profile: None,
      is_async: false,
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

  pub fn add_outgoing_connection(&mut self, connection_id: ConnectionId) {
    self.outgoing_connections.insert(connection_id);
  }

  pub fn incoming_connections_unordered<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Result<impl Iterator<Item = &'m ModuleGraphConnection>> {
    let result = self
      .incoming_connections
      .iter()
      .map(|connection_id| {
        module_graph
          .connection_by_connection_id(connection_id)
          .ok_or_else(|| {
            internal_error!(
              "connection_id_to_connection does not have connection_id: {connection_id:?}"
            )
          })
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter();

    Ok(result)
  }

  pub fn outgoing_connections_unordered<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Result<impl Iterator<Item = &'m ModuleGraphConnection>> {
    let result = self
      .outgoing_connections
      .iter()
      .map(|connection_id| {
        module_graph
          .connection_by_connection_id(connection_id)
          .ok_or_else(|| {
            internal_error!(
              "connection_id_to_connection does not have connection_id: {connection_id:?}"
            )
          })
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter();

    Ok(result)
  }

  pub fn depended_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleIdentifier> {
    if IS_NEW_TREESHAKING.load(std::sync::atomic::Ordering::Relaxed) {
      self
        .outgoing_connections_unordered(module_graph)
        .expect("should have outgoing connections")
        .filter_map(|con: &ModuleGraphConnection| {
          // TODO: runtime opt
          let active_state = con.get_active_state(module_graph, None);
          // dbg!(
          //   &con,
          //   active_state,
          //   &module_graph
          //     .dependency_by_id(&con.dependency_id)
          //     .and_then(|dep| dep
          //       .as_module_dependency()
          //       .map(|item| item.dependency_debug_name()))
          // );
          match active_state {
            crate::ConnectionState::Bool(false) => None,
            _ => Some(con.dependency_id),
          }
        })
        .filter(|id| {
          let dep = module_graph.dependency_by_id(id).expect("should have id");
          if let Some(dep) = dep.as_module_dependency() {
            return module_graph.get_parent_block(id).is_none() && !dep.weak();
          } else if dep.as_context_dependency().is_some() {
            return module_graph.get_parent_block(id).is_none();
          }

          false
        })
        .filter_map(|id| module_graph.module_identifier_by_dependency_id(&id))
        .collect()
    } else {
      self
        .all_dependencies
        .iter()
        .filter(|id| {
          let dep = module_graph.dependency_by_id(id).expect("should have id");
          if let Some(dep) = dep.as_module_dependency() {
            return module_graph.get_parent_block(id).is_none() && !dep.weak();
          } else if dep.as_context_dependency().is_some() {
            return module_graph.get_parent_block(id).is_none();
          }
          false
        })
        .filter_map(|id| module_graph.module_identifier_by_dependency_id(id))
        .collect()
    }
    // dbg!(&self.module_identifier);
  }

  pub fn dynamic_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<(&'a ModuleIdentifier, Option<&'a GroupOptions>)> {
    self
      .all_dependencies
      .iter()
      .filter_map(|id| {
        let dep = module_graph.dependency_by_id(id).expect("should have id");
        let id = if let Some(dep) = dep.as_module_dependency() {
          dep.id()
        } else if let Some(dep) = dep.as_context_dependency() {
          dep.id()
        } else {
          return None;
        };

        let Some(block_id) = module_graph.get_parent_block(id) else {
          return None;
        };
        let module = module_graph
          .module_identifier_by_dependency_id(id)
          .expect("should have a module here");
        let Some(block) = module_graph.block_by_id(&block_id) else {
          return None;
        };
        Some((module, block.get_group_options()))
      })
      .collect()
  }

  pub fn all_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<&'a ModuleIdentifier> {
    self
      .all_dependencies
      .iter()
      .filter_map(|id| module_graph.module_identifier_by_dependency_id(id))
      .collect()
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

  pub fn get_exports_argument(&self) -> ExportsArgument {
    self
      .build_meta
      .as_ref()
      .map(|m| m.exports_argument)
      .unwrap_or_default()
  }

  pub fn get_module_argument(&self) -> ModuleArgument {
    self
      .build_meta
      .as_ref()
      .map(|m| m.module_argument)
      .unwrap_or_default()
  }

  pub fn get_exports_type(&self, strict: bool) -> ExportsType {
    if let Some((export_type, default_object)) = self
      .build_meta
      .as_ref()
      .map(|m| (&m.exports_type, &m.default_object))
    {
      match export_type {
        BuildMetaExportsType::Flagged => {
          if strict {
            ExportsType::DefaultWithNamed
          } else {
            ExportsType::Namespace
          }
        }
        BuildMetaExportsType::Namespace => ExportsType::Namespace,
        BuildMetaExportsType::Default => match default_object {
          BuildMetaDefaultObject::Redirect => ExportsType::DefaultWithNamed,
          BuildMetaDefaultObject::RedirectWarn => {
            if strict {
              ExportsType::DefaultOnly
            } else {
              ExportsType::DefaultWithNamed
            }
          }
          BuildMetaDefaultObject::False => ExportsType::DefaultOnly,
        },
        BuildMetaExportsType::Dynamic => {
          if strict {
            ExportsType::DefaultWithNamed
          } else {
            // TODO check target
            ExportsType::Dynamic
          }
        }
        // algin to undefined
        BuildMetaExportsType::Unset => {
          if strict {
            ExportsType::DefaultWithNamed
          } else {
            ExportsType::Dynamic
          }
        }
      }
    } else if strict {
      ExportsType::DefaultWithNamed
    } else {
      ExportsType::Dynamic
    }
  }

  pub fn get_strict_harmony_module(&self) -> bool {
    self
      .build_meta
      .as_ref()
      .map(|m| m.strict_harmony_module)
      .unwrap_or(false)
  }
}
