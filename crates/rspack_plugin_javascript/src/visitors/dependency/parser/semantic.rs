use swc_next_ecma_ast::{BindingIdentifier, NodeId};

use super::{JavascriptParser, NameInfo, RootName};
use crate::visitors::ExprRef;

impl JavascriptParser<'_> {
  pub(crate) fn in_semantic_scope<T>(&mut self, node: NodeId, f: impl FnOnce(&mut Self) -> T) -> T {
    let previous = self.definitions_db.enter_semantic_scope(self.ast, node);
    let result = f(self);
    self.definitions_db.leave_semantic_scope(previous);
    result
  }

  pub(super) fn define_variable_identifier(&mut self, identifier: BindingIdentifier) {
    self.definitions_db.define_identifier(self.ast, identifier);
  }

  pub(super) fn pre_define_variable_identifier(&mut self, identifier: BindingIdentifier) {
    self
      .definitions_db
      .pre_define_identifier(self.ast, identifier);
  }

  pub(super) fn activate_semantic_scope_bindings(&mut self) {
    self.definitions_db.activate_scope_bindings(self.ast);
  }

  pub(super) fn define_function_declaration(&mut self, identifier: BindingIdentifier) {
    self
      .definitions_db
      .define_function_declaration(self.ast, identifier);
  }

  pub(super) fn get_name_info_from_root(&mut self, root: ExprRef) -> Option<NameInfo<'_>> {
    let ExprRef::Ident(identifier) = root else {
      return self.get_name_info_from_variable(root.get_root_name(self.ast.ast)?);
    };
    let Some(state) = self.definitions_db.resolve_identifier(self.ast, identifier) else {
      let ast = self.ast.ast;
      return Some(NameInfo {
        name: ast.get_utf8(identifier.name(ast)),
        info: None,
      });
    };
    let info = self.definitions_db.expect_get_variable(state);
    if !info.is_free() && !info.is_tagged() {
      return None;
    }
    Some(NameInfo {
      name: info.name?,
      info: Some(info),
    })
  }
}
