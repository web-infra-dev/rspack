use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyId,
  DependencyTemplate, DynamicDependencyTemplate, DynamicDependencyTemplateType, TemplateContext,
  TemplateReplaceSource,
};

use super::local_module::LocalModule;

#[cacheable]
#[derive(Debug, Clone)]
pub struct LocalModuleDependency {
  id: DependencyId,
  local_module: LocalModule,
  range: Option<(u32, u32)>,
  call_new: bool,
}

impl LocalModuleDependency {
  pub fn new(local_module: LocalModule, range: Option<(u32, u32)>, call_new: bool) -> Self {
    Self {
      id: DependencyId::new(),
      local_module,
      range,
      call_new,
    }
  }

  pub fn get_variable_name(&self) -> String {
    self.local_module.variable_name()
  }
}

#[cacheable_dyn]
impl Dependency for LocalModuleDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

#[cacheable_dyn]
impl DependencyTemplate for LocalModuleDependency {
  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    Some(LocalModuleDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for LocalModuleDependency {}

impl AsContextDependency for LocalModuleDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct LocalModuleDependencyTemplate;

impl LocalModuleDependencyTemplate {
  pub fn template_type() -> DynamicDependencyTemplateType {
    DynamicDependencyTemplateType::CustomType("LocalModuleDependency")
  }
}

impl DynamicDependencyTemplate for LocalModuleDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<LocalModuleDependency>()
      .expect("LocalModuleDependencyTemplate should only be used for LocalModuleDependency");

    if let Some(range) = dep.range {
      let module_instance = if dep.call_new {
        format!(
          "new (function () {{ return {}; }})()",
          dep.local_module.variable_name()
        )
      } else {
        dep.local_module.variable_name()
      };
      source.replace(range.0, range.1, &module_instance, None);
    }
  }
}
