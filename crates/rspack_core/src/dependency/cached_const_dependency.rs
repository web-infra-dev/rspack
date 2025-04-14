use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_util::ext::DynHash;

use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
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
impl DependencyCodeGeneration for CachedConstDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CachedConstDependencyTemplate::template_type())
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

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CachedConstDependencyTemplate;

impl CachedConstDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("CachedConstDependency")
  }
}

impl DependencyTemplate for CachedConstDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CachedConstDependency>()
      .expect("CachedConstDependencyTemplate should be used for CachedConstDependency");

    code_generatable_context.init_fragments.push(
      NormalInitFragment::new(
        format!("var {} = {};\n", dep.identifier, dep.content),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::Const(dep.identifier.to_string()),
        None,
      )
      .boxed(),
    );
    source.replace(dep.start, dep.end, &dep.identifier, None);
  }
}
