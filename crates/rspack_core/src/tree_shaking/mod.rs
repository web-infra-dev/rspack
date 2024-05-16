use bitflags;
use once_cell::sync::Lazy;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::ast::ModuleItem;

use self::visitor::{OptimizeAnalyzeResult, SymbolRef};

pub mod analyzer;
pub mod asset_module;
pub mod debug_helper;
pub mod js_module;
pub mod symbol;
pub mod symbol_graph;
pub mod utils;
pub mod visitor;
pub mod webpack_ext;

mod test;

#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol_ref: HashSet<SymbolRef>,
  pub analyze_results: IdentifierMap<OptimizeAnalyzeResult>,
  pub bail_out_module_identifiers: IdentifierMap<BailoutFlag>,
  pub side_effects_free_modules: IdentifierSet,
  pub module_item_map: IdentifierMap<Vec<ModuleItem>>,
  pub include_module_ids: IdentifierSet,
}

const ANALYZE_LOGGING: bool = true;
static CARE_MODULE_ID_FROM_ENV: Lazy<Vec<String>> = Lazy::new(|| {
  let cwd = std::env::current_dir().expect("");
  match &std::env::var("CARE_ID") {
    Ok(relative_path) => {
      let ab_path = cwd.join(relative_path);
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
  if CARED_MODULE_ID.is_empty() {
    return true;
  }

  CARED_MODULE_ID
    .iter()
    .any(|module_id| id.as_ref().contains(module_id))
    || CARE_MODULE_ID_FROM_ENV
      .iter()
      .any(|module_id| id.as_ref().contains(module_id))
}

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy)]
  pub struct BailoutFlag: u8 {
      const COMMONJS_REQUIRE = 1 << 1;
      const COMMONJS_EXPORTS = 1 << 2;
      const DYNAMIC_IMPORT = 1 << 3;
      const CONTEXT_MODULE = 1 << 4;
      const CONTAINER_EXPOSED = 1 << 5;
      const BUILDTIME_EXECUTION = 1 << 6;
  }
}

bitflags::bitflags! {
  #[derive(Debug)]
  pub struct ModuleUsedType: u8 {
    const DIRECT = 1 << 0;
    const REEXPORT = 1 << 1;
    const EXPORT_ALL = 1 << 2;
    const INDIRECT = 1 << 3;
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SideEffectType {
  Configuration(bool),
  Analyze(bool),
}

impl Default for SideEffectType {
  fn default() -> Self {
    SideEffectType::Analyze(true)
  }
}
