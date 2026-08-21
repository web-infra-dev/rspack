use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_hash::{RspackHash, RspackHasher};

use super::DependencyRange;
use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum CachedConstDependencyPlace {
  #[default]
  Module = 10,
  Chunk = 20,
}

impl CachedConstDependencyPlace {
  fn order(self) -> i32 {
    self as i32
  }
}

impl RspackHash for CachedConstDependencyPlace {
  fn hash(&self, state: &mut RspackHasher) {
    self.order().hash(state);
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CachedConstDependency {
  pub place: CachedConstDependencyPlace,
  pub identifier: Box<str>,
  pub range: Option<DependencyRange>,
  pub content: Box<str>,
}

impl CachedConstDependency {
  pub fn new(range: DependencyRange, identifier: Box<str>, content: Box<str>) -> Self {
    Self::new_with_place(
      range,
      identifier,
      content,
      CachedConstDependencyPlace::Module,
    )
  }

  pub fn new_with_place(
    range: DependencyRange,
    identifier: Box<str>,
    content: Box<str>,
    place: CachedConstDependencyPlace,
  ) -> Self {
    Self {
      place,
      range: Some(range),
      identifier,
      content,
    }
  }

  pub fn new_without_replacement(
    identifier: Box<str>,
    content: Box<str>,
    place: CachedConstDependencyPlace,
  ) -> Self {
    Self {
      place,
      range: None,
      identifier,
      content,
    }
  }
}

impl RspackHash for CachedConstDependency {
  fn hash(&self, state: &mut RspackHasher) {
    self.place.hash(state);
    self.identifier.hash(state);
    match self.range {
      Some(range) => range.hash(state),
      None => state.write(b"null"),
    }
    self.content.hash(state);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CachedConstDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CachedConstDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
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

    match dep.place {
      CachedConstDependencyPlace::Module => {
        code_generatable_context.init_fragments.push(
          NormalInitFragment::new(
            format!("var {} = {};\n", dep.identifier, dep.content),
            InitFragmentStage::StageConstants,
            dep.place.order(),
            InitFragmentKey::Const(dep.identifier.to_string()),
            None,
          )
          .boxed(),
        );
      }
      CachedConstDependencyPlace::Chunk => {
        code_generatable_context.chunk_init_fragments().push(
          NormalInitFragment::new(
            format!("var {} = {};\n", dep.identifier, dep.content),
            InitFragmentStage::StageConstants,
            dep.place.order(),
            InitFragmentKey::Const(dep.identifier.to_string()),
            None,
          )
          .boxed(),
        );
      }
    }

    if let Some(range) = dep.range {
      source.replace(range.start, range.end, dep.identifier.to_string(), None);
    }
  }
}
