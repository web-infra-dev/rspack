use bitflags;
use hashbrown::{HashMap, HashSet};
use rspack_symbol::{IndirectTopLevelSymbol, Symbol};

use self::visitor::TreeShakingResult;
use crate::ModuleIdentifier;

pub mod utils;
pub mod visitor;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol: HashSet<Symbol>,
  pub used_indirect_symbol: HashSet<IndirectTopLevelSymbol>,
  pub analyze_results: HashMap<ModuleIdentifier, TreeShakingResult>,
  pub bail_out_module_identifiers: HashMap<ModuleIdentifier, BailoutFlog>,
}
const DISABLE_ANALYZE_LOGGING: bool = true;
pub static CARED_MODULE_ID: &[&str] = &[];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if !DISABLE_ANALYZE_LOGGING {
    return false;
  }
  if CARED_MODULE_ID.is_empty() {
    return true;
  }
  CARED_MODULE_ID
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
