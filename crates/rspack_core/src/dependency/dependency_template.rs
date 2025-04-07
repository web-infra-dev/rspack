use std::fmt::Debug;

use dyn_clone::{clone_trait_object, DynClone};
use rspack_cacheable::cacheable_dyn;
use rspack_sources::{BoxSource, ReplaceSource};
use rspack_util::ext::AsAny;

use crate::{
  ChunkInitFragments, CodeGenerationData, Compilation, ConcatenationScope, Module,
  ModuleInitFragments, RuntimeGlobals, RuntimeSpec,
};

pub struct TemplateContext<'a, 'b, 'c> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub init_fragments: &'a mut ModuleInitFragments<'b>,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'c mut ConcatenationScope>,
  pub data: &'a mut CodeGenerationData,
}

impl TemplateContext<'_, '_, '_> {
  pub fn chunk_init_fragments(&mut self) -> &mut ChunkInitFragments {
    let data_fragments = self.data.get::<ChunkInitFragments>();
    if data_fragments.is_some() {
      self
        .data
        .get_mut::<ChunkInitFragments>()
        .expect("should have chunk_init_fragments")
    } else {
      self.data.insert(ChunkInitFragments::default());
      self
        .data
        .get_mut::<ChunkInitFragments>()
        .expect("should have chunk_init_fragments")
    }
  }
}

pub type TemplateReplaceSource = ReplaceSource<BoxSource>;

clone_trait_object!(DependencyTemplate);

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyTemplate.js
#[cacheable_dyn]
pub trait DependencyTemplate: Debug + DynClone + Sync + Send + AsAny {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    unimplemented!()
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }

  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    None
  }
}

pub type BoxDependencyTemplate = Box<dyn DependencyTemplate>;

pub trait AsDependencyTemplate {
  fn as_dependency_template(&self) -> Option<&dyn DependencyTemplate> {
    None
  }
}

impl<T: DependencyTemplate> AsDependencyTemplate for T {
  fn as_dependency_template(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DynamicDependencyTemplateType {
  DependencyType(DependencyType),
  CustomType(String),
}

pub trait DynamicDependencyTemplate: Debug + Sync + Send {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  );
}
