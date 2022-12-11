use hashbrown::{HashMap, HashSet};
use rspack_symbol::{IndirectTopLevelSymbol, Symbol};
use ustr::Ustr;

use self::visitor::TreeShakingResult;

pub mod utils;
pub mod visitor;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol: HashSet<Symbol>,
  pub used_indirect_symbol: HashSet<IndirectTopLevelSymbol>,
  pub analyze_results: HashMap<Ustr, TreeShakingResult>,
  pub bail_out_module_identifiers: HashMap<Ustr, BailoutReason>,
}
const DISABLE_ANALYZE_LOGGING: bool = false;
pub static CARED_MODULE_ID: &[&str] = &[
  // "common/temp/node_modules/.pnpm/@universe-design+react@2.20.4_kmhvk2unyuwyyku4khnxoqgdjq/node_modules/@universe-design/react/es/badge/SVGTextBadge.js",
  // "common/temp/node_modules/.pnpm/@universe-design+react@2.20.4_kmhvk2unyuwyyku4khnxoqgdjq/node_modules/@universe-design/react/es/badge/utils.js"
  // "/Users/bytedance/Documents/bytedance/monorepo/common/temp/node_modules/.pnpm/antd@4.6.6_wcqkhtmu7mswc6yz4uyexck3ty/node_modules/antd/es/layout/index.js",

  // "/Users/bytedance/Documents/bytedance/monorepo/common/temp/node_modules/.pnpm/antd@4.6.6_wcqkhtmu7mswc6yz4uyexck3ty/node_modules/antd/es"
];

pub fn debug_care_module_id<T: AsRef<str>>(id: T) -> bool {
  if DISABLE_ANALYZE_LOGGING == false {
    return false;
  }
  if CARED_MODULE_ID.is_empty() {
    return true;
  }
  CARED_MODULE_ID
    .iter()
    .any(|module_id| id.as_ref().contains(module_id))
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
