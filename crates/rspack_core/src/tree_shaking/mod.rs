use bitflags;
use once_cell::sync::Lazy;
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::ast::ModuleItem;

use self::visitor::{SymbolRef, TreeShakingResult};
use crate::{IdentifierMap, IdentifierSet};

pub mod symbol_graph;
pub mod utils;
pub mod visitor;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol_ref: HashSet<SymbolRef>,
  pub analyze_results: IdentifierMap<TreeShakingResult>,
  pub bail_out_module_identifiers: IdentifierMap<BailoutFlog>,
  pub side_effects_free_modules: IdentifierSet,
  pub module_item_map: IdentifierMap<Vec<ModuleItem>>,
}
const ANALYZE_LOGGING: bool = true;
static CARE_MODULE_ID_FROM_ENV: Lazy<Vec<String>> = Lazy::new(|| {
  let log = std::env::current_dir().expect("");
  match &std::env::var("CARE_ID") {
    Ok(relative_path) => {
      let ab_path = log.join(relative_path);
      let file = std::fs::read_to_string(ab_path).expect("Failed to read target file into string");
      file
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
    }
    Err(_) => vec![],
  }
});
pub static CARED_MODULE_ID: &[&str] = &[];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if !ANALYZE_LOGGING {
    return false;
  }

  // .chain()
  CARED_MODULE_ID
    .iter()
    .any(|module_id| id.as_ref().contains(module_id))
    || CARE_MODULE_ID_FROM_ENV
      .iter()
      .any(|module_id| id.as_ref().contains(module_id))
}

bitflags::bitflags! {
  pub struct BailoutFlog: u8 {
      const HELPER = 1 << 0;
      const COMMONJS_REQUIRE = 1 << 1;
      const COMMONJS_EXPORTS = 1 << 2;
      const DYNAMIC_IMPORT = 1 << 3;
  }
}

bitflags::bitflags! {
  pub struct ModuleUsedType: u8 {
    const DIRECT = 1 << 0;
    const REEXPORT = 1 << 1;
    const EXPORT_ALL = 1 << 2;
    const INDIRECT = 1 << 3;
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SideEffect {
  Configuration(bool),
  Analyze(bool),
}
