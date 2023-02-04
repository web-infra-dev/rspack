use bitflags;
use rspack_symbol::{IndirectTopLevelSymbol, Symbol};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::ast::{ModuleDecl, ModuleItem};

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
pub static CARED_MODULE_ID: &[&str] = &[
  "/Users/bytedance/Documents/rspack/rspack/node_modules/@arco-design/web-react/node_modules/react-transition-group/esm/CSSTransition.js",
  "/Users/bytedance/Documents/rspack/rspack/node_modules/prop-types/index.js"
];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if !ANALYZE_LOGGING {
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

bitflags::bitflags! {
  pub struct ModuleUsedType: u8 {
    const DIRECT = 1 << 0;
    const REEXPORT = 1 << 1;
    const EXPORT_STAR = 1 << 2;
    const INDIRECT = 1 << 3;
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SideEffect {
  Configuration(bool),
  Analyze(bool),
}
