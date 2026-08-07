use std::fmt::Debug;

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::cacheable_dyn;
use rspack_hash::RspackHasher;
use rspack_sources::ReplaceSource;
use rspack_util::ext::AsAny;

use crate::{
  ChunkInitFragments, CodeGenerationData, Compilation, ConcatenationScope, DependencyRange,
  DependencyType, Module, ModuleCodeTemplate, ModuleInitFragments, RuntimeSpec,
};

pub struct TemplateContext<'a, 'b, 'c> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub init_fragments: &'a mut ModuleInitFragments<'b>,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'c mut ConcatenationScope>,
  pub data: &'a mut CodeGenerationData,
  pub runtime_template: &'a mut ModuleCodeTemplate,
}

impl TemplateContext<'_, '_, '_> {
  pub fn faster_concatenation_scope(&mut self) -> Option<&mut ConcatenationScope> {
    self
      .concatenation_scope
      .as_deref_mut()
      .filter(|scope| scope.is_faster_module_concatenation())
  }

  pub fn ensure_generated_top_level_symbol(&mut self, preferred_name: impl Into<String>) -> String {
    let preferred_name = preferred_name.into();
    self
      .ensure_generated_top_level_symbol_in_scope(&preferred_name)
      .unwrap_or(preferred_name)
  }

  pub fn ensure_generated_top_level_symbol_in_scope(
    &mut self,
    preferred_name: &str,
  ) -> Option<String> {
    self.faster_concatenation_scope().map(|scope| {
      scope
        .ensure_generated_top_level_symbol(preferred_name)
        .to_string()
    })
  }

  pub fn remove_original_range(&mut self, range: DependencyRange) {
    if let Some(scope) = self.faster_concatenation_scope() {
      scope.remove_original_range(range);
    }
  }

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
    _hasher: &mut RspackHasher,
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
