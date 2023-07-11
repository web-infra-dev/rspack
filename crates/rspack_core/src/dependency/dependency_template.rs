use std::fmt::Debug;

use rspack_sources::{BoxSource, ReplaceSource};

use crate::{Compilation, InitFragment, Module, RuntimeGlobals};

pub struct TemplateContext<'a> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub init_fragments: &'a mut Vec<InitFragment>,
}

pub type TemplateReplaceSource = ReplaceSource<BoxSource>;

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyTemplate.js
pub trait DependencyTemplate: Debug + Sync + Send {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  );
}
