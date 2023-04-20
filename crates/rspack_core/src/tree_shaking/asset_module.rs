use super::{analyzer::OptimizeAnalyzer, visitor::OptimizeAnalyzeResult};
use crate::ModuleIdentifier;

pub struct AssetModule {
  module_identifier: ModuleIdentifier,
}

impl AssetModule {
  fn new(module_identifier: ModuleIdentifier) -> Self {
    Self { module_identifier }
  }
}

impl OptimizeAnalyzer for AssetModule {
  fn analyze(&self, compilation: &crate::Compilation) -> OptimizeAnalyzeResult {
    let result = OptimizeAnalyzeResult::default();
    result
  }
}
