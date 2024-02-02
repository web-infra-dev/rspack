use crate::{
  AsDependency, DependencyTemplate, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct ConstDependency {
  pub start: u32,
  pub end: u32,
  pub content: Box<str>,
  pub runtime_requirements: Option<RuntimeGlobals>,
}

impl ConstDependency {
  pub fn new(
    start: u32,
    end: u32,
    content: Box<str>,
    runtime_requirements: Option<RuntimeGlobals>,
  ) -> Self {
    Self {
      start,
      end,
      content,
      runtime_requirements,
    }
  }
}

impl DependencyTemplate for ConstDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    if let Some(runtime_requirements) = &self.runtime_requirements {
      code_generatable_context
        .runtime_requirements
        .insert(*runtime_requirements);
    }
    source.replace(self.start, self.end, self.content.as_ref(), None);
  }

  fn dependency_id(&self) -> Option<crate::DependencyId> {
    None
  }
}

impl AsDependency for ConstDependency {}
