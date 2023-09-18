use super::{analyzer::OptimizeAnalyzer, visitor::OptimizeAnalyzeResult, SideEffectType};
use crate::ModuleIdentifier;

pub struct AssetModule {
  module_identifier: ModuleIdentifier,
}

impl AssetModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    Self { module_identifier }
  }
}

impl OptimizeAnalyzer for AssetModule {
  fn analyze(&self) -> OptimizeAnalyzeResult {
    let mut result = OptimizeAnalyzeResult::default();
    result.side_effects = SideEffectType::Configuration(true);
    result.module_identifier = self.module_identifier;
    result
  }
}
