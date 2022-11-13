use hashbrown::{HashMap, HashSet};
use rspack_symbol::Symbol;
use ustr::Ustr;

use self::visitor::TreeShakingResult;

pub mod utils;
pub mod visitor;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol: HashSet<Symbol>,
  pub analyze_results: HashMap<Ustr, TreeShakingResult>,
  pub bail_out_module_identifiers: HashMap<Ustr, BailoutReason>,
}

pub static CARE_MODULE_ID: &[&str] = &[
  "/home/victor/Documents/rspack/rspack/examples/arco-pro/src/layout.tsx",
  "/home/victor/Documents/rspack/rspack/examples/arco-pro/src/utils/getUrlParams.ts",
];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if CARE_MODULE_ID.len() == 0 {
    return true;
  }
  CARE_MODULE_ID.contains(&id.as_ref())
}

#[derive(Debug, Clone, Copy)]
pub enum BailoutReason {
  Helper,
  CommonjsRequire,
  CommonjsExports,
  DynamicImport,
  /// TODO: remove this reason after we visit commonjs
  ExtendBailout,
}
