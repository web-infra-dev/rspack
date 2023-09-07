// use rspack_core::Plugin;
// use rspack_error::Result;
use swc_core::common::{Span, Spanned, SyntaxContext, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

#[derive(Debug)]
pub struct SideEffectsFlagPluginVisitor {
  unresolved_ctxt: SyntaxContext,
  side_effects_span: Option<Span>,
  is_top_level: bool,
}

#[derive(Debug)]
pub struct SyntaxContextInfo {
  unresolved_ctxt: SyntaxContext,
}

impl SyntaxContextInfo {
  pub fn new(unresolved_ctxt: SyntaxContext) -> Self {
    Self { unresolved_ctxt }
  }
}

impl SideEffectsFlagPluginVisitor {
  pub fn new(mark_info: SyntaxContextInfo) -> Self {
    Self {
      unresolved_ctxt: mark_info.unresolved_ctxt,
      side_effects_span: None,
      is_top_level: true,
    }
  }
}

impl Visit for SideEffectsFlagPluginVisitor {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
  }

  fn visit_module(&mut self, node: &Module) {
    for module_item in &node.body {
      if !is_import_decl(module_item) {
        module_item.visit_with(self);
      }
    }
  }

  fn visit_script(&mut self, node: &Script) {
    for stmt in &node.body {
      stmt.visit_with(self);
    }
  }

  fn visit_stmt(&mut self, node: &Stmt) {
    if !self.is_top_level {
      return;
    }
    self.analyze_stmt_side_effects(node);
    node.visit_children_with(self);
  }

  fn visit_class_member(&mut self, node: &ClassMember) {
    if let Some(key) = node.class_key() && key.is_computed() {
      key.visit_with(self);
    }

    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }
}

impl SideEffectsFlagPluginVisitor {
  /// If we find a stmt that has side effects, we will skip the rest of the stmts.
  /// And mark the module as having side effects.
  fn analyze_stmt_side_effects(&mut self, ele: &Stmt) {
    if self.side_effects_span.is_none() {
      match ele {
        Stmt::If(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span);
          }
        }
        Stmt::While(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span);
          }
        }
        Stmt::DoWhile(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span);
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
            self.side_effects_span = Some(stmt.span);
            return;
          }

          let pure_test = match stmt.test {
            Some(box ref test) => is_pure_expression(test, self.unresolved_ctxt),
            None => true,
          };

          if !pure_test {
            self.side_effects_span = Some(stmt.span);
            return;
          }

          let pure_update = match stmt.update {
            Some(ref expr) => is_pure_expression(expr, self.unresolved_ctxt),
            None => true,
          };

          if !pure_update {
            self.side_effects_span = Some(stmt.span);
          }
        }
        Stmt::Expr(stmt) => {
          if !is_pure_expression(&stmt.expr, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span);
          }
        }
        Stmt::Switch(stmt) => {
          if !is_pure_expression(&stmt.discriminant, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span);
          }
        }
        Stmt::Decl(stmt) => {
          if !is_pure_decl(stmt, self.unresolved_ctxt) {
            self.side_effects_span = Some(stmt.span());
          }
        }
        Stmt::Empty(_) => {}
        Stmt::Labeled(_) => {}
        Stmt::Block(_) => {}
        _ => self.side_effects_span = Some(ele.span()),
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

pub trait ClassKey {
  fn class_key(&self) -> Option<&PropName>;
}

impl ClassKey for ClassMember {
  fn class_key(&self) -> Option<&PropName> {
    match self {
      ClassMember::Constructor(c) => Some(&c.key),
      ClassMember::Method(m) => Some(&m.key),
      ClassMember::PrivateMethod(_) => None,
      ClassMember::ClassProp(c) => Some(&c.key),
      ClassMember::PrivateProp(_) => None,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => None,
      ClassMember::StaticBlock(_) => None,
      ClassMember::AutoAccessor(a) => match a.key {
        Key::Private(_) => None,
        Key::Public(ref public) => Some(public),
      },
    }
  }
}

#[derive(Debug)]
pub struct SideEffectsFlagPlugin {}

impl SideEffectsFlagPlugin {
  pub fn new() -> Self {
    Self {}
  }
}
