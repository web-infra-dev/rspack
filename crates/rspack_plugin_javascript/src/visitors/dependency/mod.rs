mod context_dependency_helper;
mod context_helper;
mod parser;
mod util;

use rustc_hash::FxHashMap;
use swc_core::ecma::atoms::Atom;

pub use self::context_dependency_helper::create_context_dependency;
pub use self::context_helper::{scanner_context_module, ContextModuleScanResult};
pub use self::parser::{CallExpressionInfo, CallHooksName, ExportedVariableInfo};
pub use self::parser::{JavascriptParser, MemberExpressionInfo, TagInfoData, TopLevelScope};
pub use self::util::*;
use crate::dependency::Specifier;

#[derive(Debug)]
pub struct ImporterReferenceInfo {
  pub request: Atom,
  pub specifier: Specifier,
  pub names: Option<Atom>,
  pub source_order: i32,
}

impl ImporterReferenceInfo {
  pub fn new(request: Atom, specifier: Specifier, names: Option<Atom>, source_order: i32) -> Self {
    Self {
      request,
      specifier,
      names,
      source_order,
    }
  }
}

pub type ImportMap = FxHashMap<swc_core::ecma::ast::Id, ImporterReferenceInfo>;
