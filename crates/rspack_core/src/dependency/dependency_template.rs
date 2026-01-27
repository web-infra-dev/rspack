use std::fmt::Debug;

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::cacheable_dyn;
use rspack_sources::ReplaceSource;
use rspack_util::ext::AsAny;

use crate::{
  ChunkInitFragments, CodeGenerationData, Compilation, ConcatenationScope, DependencyType, Module,
  ModuleCodegenRuntimeTemplate, ModuleInitFragments, RuntimeGlobals, RuntimeSpec,
};

pub struct TemplateContext<'a, 'b, 'c> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub init_fragments: &'a mut ModuleInitFragments<'b>,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'c mut ConcatenationScope>,
  pub data: &'a mut CodeGenerationData,
  pub runtime_template: &'a mut ModuleCodegenRuntimeTemplate,
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

pub type TemplateReplaceSource = ReplaceSource;

clone_trait_object!(DependencyCodeGeneration);

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyCodeGeneration.js
#[cacheable_dyn]
pub trait DependencyCodeGeneration: Debug + DynClone + Sync + Send + AsAny {
  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }

  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    None
  }
}

pub type BoxDependencyTemplate = Box<dyn DependencyCodeGeneration>;

pub trait AsDependencyCodeGeneration {
  fn as_dependency_code_generation(&self) -> Option<&dyn DependencyCodeGeneration> {
    None
  }
}

impl<T: DependencyCodeGeneration> AsDependencyCodeGeneration for T {
  fn as_dependency_code_generation(&self) -> Option<&dyn DependencyCodeGeneration> {
    Some(self)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DependencyTemplateType {
  Dependency(DependencyType),
  Custom(&'static str),
}

pub trait DependencyTemplate: Debug + Sync + Send {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut ReplaceSource,
    code_generatable_context: &mut TemplateContext,
  );
}
