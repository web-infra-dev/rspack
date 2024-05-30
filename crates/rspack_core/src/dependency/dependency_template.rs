use std::fmt::Debug;

use dyn_clone::{clone_trait_object, DynClone};
use rspack_error::Diagnostic;
use rspack_sources::{BoxSource, ReplaceSource};
use rspack_util::ext::AsAny;

use crate::{
  AsDependency, CodeGenerationData, Compilation, ConcatenationScope, DependencyId, Module,
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
  pub diagnostics: &'a mut Vec<Diagnostic>,
}

pub type TemplateReplaceSource = ReplaceSource<BoxSource>;

clone_trait_object!(DependencyTemplate);

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyTemplate.js
pub trait DependencyTemplate: Debug + DynClone + Sync + Send + AsDependency + AsAny {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  );

  fn dependency_id(&self) -> Option<DependencyId>;
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
