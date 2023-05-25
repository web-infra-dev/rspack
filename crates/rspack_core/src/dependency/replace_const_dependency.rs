use std::borrow::Cow;

use crate::{
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, RuntimeGlobals,
};

#[derive(Debug)]
pub struct ReplaceConstDependency {
  pub start: u32,
  pub end: u32,
  pub content: Cow<'static, str>,
  pub runtime_requirements: Option<RuntimeGlobals>,
}

impl ReplaceConstDependency {
  pub fn new(
    start: u32,
    end: u32,
    content: Cow<'static, str>,
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

impl CodeReplaceSourceDependency for ReplaceConstDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    if let Some(runtime_requirements) = &self.runtime_requirements {
      code_generatable_context
        .runtime_requirements
        .add(*runtime_requirements);
    }
    source.replace(self.start, self.end, self.content.as_ref(), None);
  }
}
