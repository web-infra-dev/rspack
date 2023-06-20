use sugar_path::SugarPath;

use super::visitor::SideEffects;
use super::{analyzer::OptimizeAnalyzer, visitor::OptimizeAnalyzeResult, SideEffectType};
use crate::tree_shaking::visitor::get_side_effects_from_package_json;
use crate::{Compilation, FactoryMeta, ModuleIdentifier};

pub struct AssetModule {
  module_identifier: ModuleIdentifier,
}

impl AssetModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    Self { module_identifier }
  }

  fn get_side_effects_from_config(&self, compilation: &Compilation) -> Option<SideEffectType> {
    // sideEffects in module.rule has higher priority,
    // we could early return if we match a rule.
    if let Some(mgm) = compilation
      .module_graph
      .module_graph_module_by_identifier(&self.module_identifier)
      && let Some(FactoryMeta { side_effects: Some(side_effects) }) = &mgm.factory_meta
    {
      return Some(SideEffectType::Configuration(*side_effects))
    }
    None
  }
}

impl OptimizeAnalyzer for AssetModule {
  fn analyze(&self, compilation: &Compilation) -> OptimizeAnalyzeResult {
    let mut result = OptimizeAnalyzeResult::default();
    result.side_effects = self
      .get_side_effects_from_config(compilation)
      .unwrap_or(SideEffectType::Configuration(true));
    result.module_identifier = self.module_identifier;
    result
  }
}
