use rspack_collections::IdentifierSet;
use rspack_core::{Compilation, Module};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};

use crate::{
  dependency::CssImportDependency,
  parser_and_generator::walker::{CssModuleGraphSink, CssModuleGraphWalker},
};

pub(crate) fn hash_css_modules(
  module: &dyn Module,
  compilation: &Compilation,
  hasher: &mut RspackHasher,
) {
  let mut sink = CssHashSink {
    hasher,
    visited: IdentifierSet::default(),
  };
  sink.visit_module(module);
  CssModuleGraphWalker::new(compilation)
    .walk_css_imports(&mut sink, module, true)
    .expect("CSS hash sink should not fail");
}

struct CssHashSink<'a> {
  hasher: &'a mut RspackHasher,
  visited: IdentifierSet,
}

impl<'g> CssModuleGraphSink<'g> for CssHashSink<'_> {
  type ExportValue = ();
  type State = ();

  fn visit_module(&mut self, module: &dyn Module) -> bool {
    if !self.visited.insert(module.identifier()) {
      return false;
    }
    module.build_info().hash.hash(self.hasher);
    true
  }

  fn enter_import(
    &mut self,
    _walker: &mut CssModuleGraphWalker<'g>,
    _dependency: &'g CssImportDependency,
    module: &'g dyn Module,
    _include_import_sources: bool,
  ) -> Result<Option<Self::State>> {
    Ok(self.visit_module(module).then_some(()))
  }
}
