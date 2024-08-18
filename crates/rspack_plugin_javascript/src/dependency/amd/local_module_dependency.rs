use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyId, DependencyTemplate,
  TemplateContext, TemplateReplaceSource,
};

use super::local_module::LocalModule;

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

impl Dependency for LocalModuleDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }
}

impl DependencyTemplate for LocalModuleDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    if let Some(range) = self.range {
      let module_instance = if self.call_new {
        format!(
          "new (function () {{ return {}; }})()",
          self.local_module.variable_name()
        )
      } else {
        self.local_module.variable_name()
      };
      source.replace(range.0, range.1 - 1, &module_instance, None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsModuleDependency for LocalModuleDependency {}

impl AsContextDependency for LocalModuleDependency {}
