use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_util::ext::DynHash;

use crate::{
  AsDependency, Compilation, DependencyTemplate, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
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

#[cacheable_dyn]
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
        InitFragmentKey::Const(self.identifier.to_string()),
        None,
      )
      .boxed(),
    );
    source.replace(self.start, self.end, &self.identifier, None);
  }

  fn dependency_id(&self) -> Option<crate::DependencyId> {
    None
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.identifier.dyn_hash(hasher);
    self.start.dyn_hash(hasher);
    self.end.dyn_hash(hasher);
    self.content.dyn_hash(hasher);
  }
}

impl AsDependency for CachedConstDependency {}
