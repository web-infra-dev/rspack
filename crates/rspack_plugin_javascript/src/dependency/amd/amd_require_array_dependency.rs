use std::borrow::Cow;

use itertools::Itertools;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  module_raw, AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ModuleDependency,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

use super::amd_require_item_dependency::AMDRequireItemDependency;

#[cacheable]
#[derive(Debug, Clone)]
pub enum AMDRequireArrayItem {
  String(String),
  LocalModuleDependency { local_module_variable_name: String },
  AMDRequireItemDependency { dep_id: DependencyId },
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct AMDRequireArrayDependency {
  id: DependencyId,
  deps_array: Vec<AMDRequireArrayItem>,
  range: (u32, u32),
}

impl AMDRequireArrayDependency {
  pub fn new(deps_array: Vec<AMDRequireArrayItem>, range: (u32, u32)) -> Self {
    Self {
      id: DependencyId::new(),
      deps_array,
      range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for AMDRequireArrayDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdRequireArray
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AMDRequireArrayDependency {
  fn get_content(&self, code_generatable_context: &mut TemplateContext) -> String {
    format!(
      "[{}]",
      self
        .deps_array
        .iter()
        .map(|dep| Self::content_for_dependency(dep, code_generatable_context))
        .join(", ")
    )
  }

  fn content_for_dependency<'a>(
    dep: &'a AMDRequireArrayItem,
    code_generatable_context: &mut TemplateContext,
  ) -> Cow<'a, str> {
    match dep {
      AMDRequireArrayItem::String(name) => name.into(),
      AMDRequireArrayItem::LocalModuleDependency {
        local_module_variable_name,
      } => local_module_variable_name.into(),
      AMDRequireArrayItem::AMDRequireItemDependency { dep_id } => {
        let mg = code_generatable_context.compilation.get_module_graph();
        let dep = mg
          .dependency_by_id(dep_id)
          .and_then(|dep| dep.downcast_ref::<AMDRequireItemDependency>())
          .expect("should have AMDRequireItemDependency");
        module_raw(
          code_generatable_context.compilation,
          code_generatable_context.runtime_requirements,
          dep_id,
          dep.request(),
          dep.weak(),
        )
        .into()
      }
    }
  }
}

#[cacheable_dyn]
impl DependencyTemplate for AMDRequireArrayDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let content = self.get_content(code_generatable_context);
    source.replace(self.range.0, self.range.1, &content, None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsModuleDependency for AMDRequireArrayDependency {}

impl AsContextDependency for AMDRequireArrayDependency {}
