use std::collections::{HashMap, HashSet};

use linked_hash_map::LinkedHashMap;
use smol_str::SmolStr;
use swc_atoms::JsWord;
use swc_common::util::take::Take;

use crate::ResolvedId;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DynImportDesc {
  pub argument: JsWord,
  // pub id: Option<JsWord>,
}

pub struct JsModule {
  pub exec_order: usize,
  pub id: SmolStr,
  pub ast: ast::Program,
  pub dependencies: LinkedHashMap<JsWord, ()>,
  pub dyn_imports: HashSet<DynImportDesc>,
  pub is_user_defined_entry_point: bool,
  pub resolved_ids: HashMap<JsWord, ResolvedId>,
}

impl std::fmt::Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("exec_order", &self.exec_order)
      .field("id", &self.id)
      // .field("ast", &self.ast)
      .field("dependencies", &self.dependencies)
      .field("dyn_dependencies", &self.dyn_imports)
      .field(
        "is_user_defined_entry_point",
        &self.is_user_defined_entry_point,
      )
      // .field("dependency_resolver", &self.dependency_resolver)
      .finish()
  }
}

impl JsModule {
  pub fn new() -> Self {
    Self {
      exec_order: Default::default(),
      id: Default::default(),
      ast: ast::Program::Module(Take::dummy()),
      dependencies: Default::default(),
      dyn_imports: Default::default(),
      is_user_defined_entry_point: Default::default(),
      resolved_ids: Default::default(),
    }
  }
}
