use bitflags;
use once_cell::sync::Lazy;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rspack_symbol::{IndirectTopLevelSymbol, StarSymbol, Symbol};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::ast::ModuleItem;

use self::visitor::{SymbolRef, TreeShakingResult};
use crate::ModuleGraph;
pub mod debug_helper;
pub mod optimizer;
pub mod symbol_graph;
pub mod utils;
pub mod visitor;
pub mod webpack_ext;

mod test;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol_ref: HashSet<SymbolRef>,
  pub analyze_results: IdentifierMap<TreeShakingResult>,
  pub bail_out_module_identifiers: IdentifierMap<BailoutFlag>,
  pub side_effects_free_modules: IdentifierSet,
  pub module_item_map: IdentifierMap<Vec<ModuleItem>>,
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
  pub struct BailoutFlag: u8 {
      const COMMONJS_REQUIRE = 1 << 1;
      const COMMONJS_EXPORTS = 1 << 2;
      const DYNAMIC_IMPORT = 1 << 3;
      const CONTEXT_MODULE = 1 << 4;
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

pub trait ConvertModulePath {
  fn convert_module_identifier_to_module_path(self, module_graph: &ModuleGraph) -> Self;
}

impl ConvertModulePath for SymbolRef {
  fn convert_module_identifier_to_module_path(self, module_graph: &ModuleGraph) -> Self {
    match self {
      SymbolRef::Direct(direct) => {
        SymbolRef::Direct(direct.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Indirect(indirect) => {
        SymbolRef::Indirect(indirect.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Star(star) => {
        SymbolRef::Star(star.convert_module_identifier_to_module_path(module_graph))
      }
    }
  }
}

impl ConvertModulePath for Symbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    self.set_uri(
      module_graph
        .normal_module_source_path_by_identifier(&self.uri())
        .expect("Can't get module source path by identifier")
        .as_ref()
        .into(),
    );
    self
  }
}

impl ConvertModulePath for IndirectTopLevelSymbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    self.set_importer(
      module_graph
        .normal_module_source_path_by_identifier(&self.importer())
        .expect("Can't get module source path by identifier")
        .as_ref()
        .into(),
    );
    self.set_src(
      module_graph
        .normal_module_source_path_by_identifier(&self.src())
        .expect("Can't get module source path by identifier")
        .as_ref()
        .into(),
    );
    self
  }
}

impl ConvertModulePath for StarSymbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    self.set_src(
      module_graph
        .normal_module_source_path_by_identifier(&self.src())
        .expect("Can't get module source path by identifier")
        .as_ref()
        .into(),
    );
    self.set_module_ident(
      module_graph
        .normal_module_source_path_by_identifier(&self.module_ident())
        .expect("Can't get module source path by identifier")
        .as_ref()
        .into(),
    );
    self
  }
}
