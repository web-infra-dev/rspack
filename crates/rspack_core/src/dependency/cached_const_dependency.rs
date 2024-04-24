use crate::{
  AsDependency, DependencyTemplate, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  NormalInitFragment, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct CachedConstDependency {
  pub start: u32,
  pub end: u32,
  pub identifier: Box<str>,
  pub content: Box<str>,
}

impl CachedConstDependency {
  pub fn new(start: u32, end: u32, identifier: Box<str>, content: Box<str>) -> Self {
    Self {
      start,
      end,
      identifier,
      content,
    }
  }
}

impl DependencyTemplate for CachedConstDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    code_generatable_context.init_fragments.push(
      NormalInitFragment::new(
        format!("var {} = {};\n", self.identifier, self.content),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::unique(),
        None,
      )
      .boxed(),
    );
    source.replace(self.start, self.end, &self.identifier, None);
  }

  fn dependency_id(&self) -> Option<crate::DependencyId> {
    None
  }
}

impl AsDependency for CachedConstDependency {}
