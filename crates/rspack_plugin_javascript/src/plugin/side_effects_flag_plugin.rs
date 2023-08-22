use std::{collections::hash_map::Entry, collections::VecDeque, hash::Hash, path::PathBuf};

use rspack_core::ModuleIdentifier;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::common::SyntaxContext;
use swc_core::common::{util::take::Take, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::{js_word, JsWord};
use swc_core::ecma::utils::{ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use swc_node_comments::SwcComments;

#[derive(Debug)]
pub struct SideEffectsFlagPlugin {
  top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
  module_identifier: ModuleIdentifier,
  has_side_effects_stmt: bool,
}

#[derive(Debug)]
pub struct SyntaxContextInfo {
  top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
}

impl SyntaxContextInfo {
  pub fn new(top_level_ctxt: SyntaxContext, unresolved_ctxt: SyntaxContext) -> Self {
    Self {
      top_level_ctxt,
      unresolved_ctxt,
    }
  }
}

impl SideEffectsFlagPlugin {
  pub fn new(mark_info: SyntaxContextInfo, module_identifier: ModuleIdentifier) -> Self {
    Self {
      top_level_ctxt: mark_info.top_level_ctxt,
      unresolved_ctxt: mark_info.unresolved_ctxt,
      module_identifier,
      has_side_effects_stmt: false,
    }
  }
}

impl Visit for SideEffectsFlagPlugin {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
  }

  fn visit_module(&mut self, node: &Module) {
    for module_item in &node.body {
      if !is_import_decl(module_item) {
        self.analyze_module_item_side_effects(module_item);
        module_item.visit_with(self);
      }
    }
  }

  fn visit_script(&mut self, node: &Script) {
    for stmt in &node.body {
      self.analyze_stmt_side_effects(stmt);
      stmt.visit_with(self);
    }
  }

  fn visit_stmt(&mut self, node: &Stmt) {
    self.analyze_stmt_side_effects(node);
  }
}
impl SideEffectsFlagPlugin {
  fn analyze_module_item_side_effects(&mut self, ele: &ModuleItem) {
    match ele {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::ExportDecl(decl) => {
          if !is_pure_decl(&decl.decl, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        ModuleDecl::ExportDefaultDecl(decl) => {
          match decl.decl {
            DefaultDecl::Class(ref class) => {
              if !is_pure_class(&class.class, self.unresolved_ctxt) {
                self.has_side_effects_stmt = true;
              }
            }
            DefaultDecl::Fn(_) => {}
            DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
          };
        }
        ModuleDecl::ExportDefaultExpr(expr) => {
          if !is_pure_expression(&expr.expr, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        ModuleDecl::ExportAll(_)
        | ModuleDecl::Import(_)
        | ModuleDecl::ExportNamed(_)
        | ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => {}
      },
      ModuleItem::Stmt(stmt) => self.analyze_stmt_side_effects(stmt),
    }
  }

  /// If we find a stmt that has side effects, we will skip the rest of the stmts.
  /// And mark the module as having side effects.
  fn analyze_stmt_side_effects(&mut self, ele: &Stmt) {
    if !self.has_side_effects_stmt {
      match ele {
        Stmt::If(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::While(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::DoWhile(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::For(stmt) => {
          let pure_init = match stmt.init {
            Some(ref init) => match init {
              VarDeclOrExpr::VarDecl(decl) => is_pure_var_decl(decl, self.unresolved_ctxt),
              VarDeclOrExpr::Expr(expr) => is_pure_expression(expr, self.unresolved_ctxt),
            },
            None => true,
          };

          if !pure_init {
            self.has_side_effects_stmt = true;
            return;
          }

          let pure_test = match stmt.test {
            Some(box ref test) => is_pure_expression(test, self.unresolved_ctxt),
            None => true,
          };

          if !pure_test {
            self.has_side_effects_stmt = true;
            return;
          }

          let pure_update = match stmt.update {
            Some(ref expr) => is_pure_expression(expr, self.unresolved_ctxt),
            None => true,
          };

          if !pure_update {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Expr(stmt) => {
          if !is_pure_expression(&stmt.expr, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Switch(stmt) => {
          if !is_pure_expression(&stmt.discriminant, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Decl(stmt) => {
          if !is_pure_decl(stmt, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Empty(_) => {}
        Stmt::Labeled(_) => {}
        Stmt::Block(_) => {}
        _ => self.has_side_effects_stmt = true,
      };
    }
  }
}

fn is_pure_expression(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  !expr.may_have_side_effects(&ExprCtx {
    unresolved_ctxt,
    is_unresolved_ref_safe: false,
  })
}

fn is_pure_decl(stmt: &Decl, unresolved_ctxt: SyntaxContext) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(&class.class, unresolved_ctxt),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(var, unresolved_ctxt),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

fn is_pure_class(class: &Class, unresolved_ctxt: SyntaxContext) -> bool {
  if let Some(ref super_class) = class.super_class {
    if !is_pure_expression(super_class, unresolved_ctxt) {
      return false;
    }
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(ref computed) => is_pure_expression(&computed.expr, unresolved_ctxt),
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(cons) => is_pure_key(&cons.key),
      ClassMember::Method(method) => is_pure_key(&method.key),
      ClassMember::PrivateMethod(method) => {
        is_pure_expression(&Expr::PrivateName(method.key.clone()), unresolved_ctxt)
      }
      ClassMember::ClassProp(prop) => {
        is_pure_key(&prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(&Expr::PrivateName(prop.key.clone()), unresolved_ctxt)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(_) => true,
    }
  })
}

fn is_pure_var_decl(var: &VarDecl, unresolved_ctxt: SyntaxContext) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(init, unresolved_ctxt)
    } else {
      true
    }
  })
}

fn is_import_decl(module_item: &ModuleItem) -> bool {
  matches!(module_item, ModuleItem::ModuleDecl(ModuleDecl::Import(_)))
}
