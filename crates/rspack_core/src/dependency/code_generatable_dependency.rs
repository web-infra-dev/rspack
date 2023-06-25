use std::fmt::Debug;

use rspack_sources::{BoxSource, ReplaceSource};

use crate::{Compilation, InitFragment, Module, RuntimeGlobals};

pub struct CodeGeneratableContext<'a> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub init_fragments: &'a mut Vec<InitFragment>,
}

pub type CodeGeneratableSource = ReplaceSource<BoxSource>;

pub trait CodeGeneratableDependency: Debug + Sync + Send {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  );
}
