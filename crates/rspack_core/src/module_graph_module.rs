use rspack_error::{internal_error, Result};
use rustc_hash::FxHashSet as HashSet;

use crate::{
  dependency::EsmDynamicImportDependency, is_async_dependency, module_graph::ConnectionId,
  BuildInfo, BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, DependencyId,
  ExportsType, FactoryMeta, ModuleDependency, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
  ModuleIssuer, ModuleSyntax, ModuleType,
};

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
  pub dependencies: Vec<DependencyId>,
  pub(crate) pre_order_index: Option<usize>,
  pub post_order_index: Option<usize>,
  pub module_syntax: ModuleSyntax,
  pub factory_meta: Option<FactoryMeta>,
  pub build_info: Option<BuildInfo>,
  pub build_meta: Option<BuildMeta>,
}

impl ModuleGraphModule {
  pub fn new(module_identifier: ModuleIdentifier, module_type: ModuleType) -> Self {
    Self {
      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),

      issuer: ModuleIssuer::Unset,
      // exec_order: usize::MAX,
      module_identifier,
      dependencies: Default::default(),
      module_type,
      pre_order_index: None,
      post_order_index: None,
      module_syntax: ModuleSyntax::empty(),
      factory_meta: None,
      build_info: None,
      build_meta: None,
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    let c = chunk_graph.get_module_id(self.module_identifier).as_ref();
    c.expect("module id not found").as_str()
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

  // pub fn dependencies(&mut self) -> Vec<&ModuleDependency> {
  //   self
  //     .outgoing_connections_unordered()
  //     .map(|conn| &conn.dependency)
  //     .collect()
  // }

  pub fn depended_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleIdentifier> {
    self
      .dependencies
      .iter()
      .filter(|id| {
        let dep = module_graph.dependency_by_id(id).expect("should have id");
        !is_async_dependency(dep) && !dep.weak()
      })
      .filter_map(|id| module_graph.module_identifier_by_dependency_id(id))
      .collect()
  }

  pub fn dynamic_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<(&'a ModuleIdentifier, Option<&'a str>)> {
    self
      .dependencies
      .iter()
      .filter_map(|id| {
        let dep = module_graph.dependency_by_id(id).expect("should have id");
        if !is_async_dependency(dep) {
          return None;
        }
        let module = module_graph
          .module_identifier_by_dependency_id(id)
          .expect("should have a module here");

        let chunk_name = dep
          .as_ref()
          .as_any()
          .downcast_ref::<EsmDynamicImportDependency>()
          .and_then(|f| f.name.as_deref());
        Some((module, chunk_name))
      })
      .collect()
  }

  pub fn all_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<&'a ModuleIdentifier> {
    self
      .dependencies
      .iter()
      .filter_map(|id| module_graph.module_identifier_by_dependency_id(id))
      .collect()
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

  pub fn get_exports_argument(&self) -> &str {
    "exports"
    // self
    //   .build_meta
    //   .as_ref()
    //   .map(|m| m.exports_argument)
    //   .unwrap_or("exports")
  }

  pub fn get_module_argument(&self) -> &str {
    self
      .build_meta
      .as_ref()
      .map(|m| m.module_argument)
      .unwrap_or("module")
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
