use rspack_core::{AsDependencyTemplate, ModuleGraph};
use rspack_core::{
  ConnectionState, Dependency, DependencyCategory, DependencyCondition, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use super::create_resource_identifier_for_esm_dependency;

#[derive(Debug, Clone)]
pub struct HarmonyImportSideEffectDependency {
  pub request: JsWord,
  pub id: DependencyId,
  resource_identifier: String,
}

impl HarmonyImportSideEffectDependency {
  pub fn new(request: JsWord) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      request,
      resource_identifier,
    }
  }
}

impl Dependency for HarmonyImportSideEffectDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmImportSideEffect
  }
}

impl ModuleDependency for HarmonyImportSideEffectDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_condition(&self, _module_graph: &ModuleGraph) -> Option<DependencyCondition> {
    let id = self.id;
    Some(DependencyCondition::Fn(Box::new(
      move |_, _, module_graph| {
        if let Some(module) = module_graph
          .parent_module_by_dependency_id(&id)
          .and_then(|module_identifier| module_graph.module_by_identifier(&module_identifier))
        {
          module.get_side_effects_connection_state(module_graph, &mut HashSet::default())
        } else {
          ConnectionState::Bool(true)
        }
      },
    )))
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    if let Some(module) = module_graph
      .parent_module_by_dependency_id(&self.id)
      .and_then(|module_identifier| module_graph.module_by_identifier(&module_identifier))
    {
      module.get_side_effects_connection_state(module_graph, module_chain)
    } else {
      ConnectionState::Bool(true)
    }
  }
}

impl AsDependencyTemplate for HarmonyImportSideEffectDependency {}
