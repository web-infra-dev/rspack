use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsPreset, AsVec},
};
use rspack_util::json_stringify;
use swc_core::ecma::atoms::Atom;

use super::{AffectType, FactorizeInfo};
use crate::{
  AsContextDependency, AsDependencyCodeGeneration, Context, ContextMode, ContextNameSpaceObject,
  ContextOptions, ContextTypePrefix, Dependency, DependencyCategory, DependencyId, DependencyType,
  ExportsInfoArtifact, ExportsType, ExtendedReferencedExport, ImportAttributes, ModuleDependency,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleLayer, ReferencedExport, ReferencedSpecifier,
  ResourceIdentifier, RuntimeSpec, create_exports_object_referenced,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContextElementDependency {
  pub id: DependencyId,
  // TODO remove this async dependency mark
  pub options: ContextOptions,
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: Context,
  pub layer: Option<ModuleLayer>,
  pub resource_identifier: ResourceIdentifier,
  #[cacheable(with=AsOption<AsVec<AsCacheable>>)]
  pub referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  pub dependency_type: DependencyType,
  pub attributes: Option<ImportAttributes>,
  pub factorize_info: FactorizeInfo,
}

impl ContextElementDependency {
  pub fn create_resource_identifier(
    resource: &str,
    path: &str,
    attributes: Option<&ImportAttributes>,
  ) -> ResourceIdentifier {
    let mut ident = format!("context{resource}|{path}");
    if let Some(attributes) = attributes {
      ident += &json_stringify(&attributes);
    }
    ident.into()
  }
}

#[cacheable_dyn]
impl Dependency for ContextElementDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_specifiers) = &self.referenced_specifiers {
      let mut refs = vec![];
      for ReferencedSpecifier {
        names,
        is_call,
        namespace_object_as_context,
      } in referenced_specifiers
      {
        let mut names = names.as_slice();
        let mut namespace_object_as_context = *namespace_object_as_context;
        let Some(parent_module) = module_graph
          .get_parent_module(&self.id)
          .and_then(|id| module_graph.module_by_identifier(id))
          .and_then(|m| m.as_context_module())
        else {
          return create_exports_object_referenced();
        };
        let Some(imported_module) = module_graph.get_module_by_dependency_id(&self.id) else {
          return create_exports_object_referenced();
        };
        let is_strict = matches!(
          parent_module.get_context_options().namespace_object,
          ContextNameSpaceObject::Strict
        );
        let exports_type = imported_module.get_exports_type(
          module_graph,
          module_graph_cache,
          exports_info_artifact,
          is_strict,
        );

        // Force enable namespace object as context for DefaultOnly and DefaultWithNamed
        // because it's more common in cjs and json
        if matches!(
          exports_type,
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
        ) {
          namespace_object_as_context = true;
        }

        if let Some(id) = names.first()
          && id == "default"
        {
          match exports_type {
            ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
              if names.len() == 1 {
                return create_exports_object_referenced();
              }
              names = &names[1..];
            }
            ExportsType::Dynamic => {
              return create_exports_object_referenced();
            }
            _ => {}
          }
        }

        if namespace_object_as_context && *is_call {
          if names.len() == 1 {
            return create_exports_object_referenced();
          }
          // remove last one
          names = &names[..names.len().saturating_sub(1)];
        }
        refs.push(ExtendedReferencedExport::Export(ReferencedExport::new(
          names.to_vec(),
          false,
          false,
        )));
      }
      refs
    } else {
      create_exports_object_referenced()
    }
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ContextElementDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }

  fn weak(&self) -> bool {
    matches!(
      self.options.mode,
      ContextMode::AsyncWeak | ContextMode::Weak
    )
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyCodeGeneration for ContextElementDependency {}
impl AsContextDependency for ContextElementDependency {}
