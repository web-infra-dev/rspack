use std::{fmt::Debug, sync::Arc};

use rspack_cacheable::cacheable_dyn;
use rspack_hash::RspackHasher;
use rspack_sources::ReplaceSource;
use rspack_util::ext::AsAny;

use crate::{
  ChunkInitFragments, CodeGenerationData, CodeGenerationDataChunkInitFragments, Compilation,
  ConcatenationScope, DependencyType, Module, ModuleCodeTemplate, ModuleInitFragments, RuntimeSpec,
};

pub struct TemplateContext<'a, 'b> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub init_fragments: &'a mut ModuleInitFragments,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'b mut ConcatenationScope>,
  pub data: &'a mut CodeGenerationData,
  pub runtime_template: &'a mut ModuleCodeTemplate,
}

impl TemplateContext<'_, '_> {
  pub fn chunk_init_fragments(&mut self) -> &mut ChunkInitFragments {
    if !self.data.contains::<CodeGenerationDataChunkInitFragments>() {
      self
        .data
        .insert(CodeGenerationDataChunkInitFragments::default());
    }
    self
      .data
      .get_mut::<CodeGenerationDataChunkInitFragments>()
      .expect("chunk init fragments should exist")
      .inner_mut()
  }
}

pub type TemplateReplaceSource = ReplaceSource;

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyCodeGeneration.js
#[cacheable_dyn]
pub trait DependencyCodeGeneration: Debug + Sync + Send + AsAny {
  fn update_hash(
    &self,
    _hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }

  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    None
  }
}

pub type DependencyCodeGenerationRef = Arc<dyn DependencyCodeGeneration>;

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
