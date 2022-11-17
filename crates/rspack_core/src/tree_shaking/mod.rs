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

pub static CARED_MODULE_ID: &[&str] = &[
  // "/home/victor/Documents/rspack/rspack/examples/arco-pro/src/index.tsx",
  // "/home/victor/Documents/rspack/rspack/examples/basic/index.js",
  // "/home/victor/Documents/rspack/rspack/node_modules/react-redux/es/index.js",
  /* "/home/victor/Documents/rspack/rspack/node_modules/@antv/g-base/esm/index.js",
   * "/home/victor/Documents/rspack/rspack/node_modules/@antv/g-base/esm/bbox/register.js",
   * "/home/victor/Documents/rspack/rspack/node_modules/@antv/g-base/esm/bbox/index.js", */
];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if CARED_MODULE_ID.is_empty() {
    return true;
  }
  CARED_MODULE_ID.contains(&id.as_ref())
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
