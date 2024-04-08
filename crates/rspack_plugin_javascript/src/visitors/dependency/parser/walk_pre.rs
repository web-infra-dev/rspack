use std::borrow::Cow;

use rustc_hash::FxHashSet;
use swc_core::ecma::ast::{AssignExpr, BlockStmt, CatchClause, Decl, DoWhileStmt};
use swc_core::ecma::ast::{Expr, FnDecl};
use swc_core::ecma::ast::{ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt, WithStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, ObjectPat, ObjectPatProp, Stmt, WhileStmt};
use swc_core::ecma::ast::{SwitchCase, SwitchStmt, TryStmt, VarDecl, VarDeclKind, VarDeclarator};

use super::JavascriptParser;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::utils::eval;

impl<'parser> JavascriptParser<'parser> {
  pub fn pre_walk_module_declarations(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.pre_walk_module_declaration(statement);
    }
  }

  pub fn pre_walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.pre_walk_statement(statement)
    }
  }

  fn pre_walk_module_declaration(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        if self
          .plugin_drive
          .clone()
          .pre_module_declaration(self, decl)
          .unwrap_or_default()
        {
          return;
        }
        match decl {
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
          _ => {
            self.is_esm = true;
          }
        }
      }
      ModuleItem::Stmt(stmt) => self.pre_walk_statement(stmt),
    }
  }

  pub fn pre_walk_statement(&mut self, statement: &Stmt) {
    if self
      .plugin_drive
      .clone()
      .pre_statement(self, statement)
      .unwrap_or_default()
    {
      return;
    }

    match statement {
      Stmt::Block(stmt) => self.pre_walk_block_statement(stmt),
      Stmt::DoWhile(stmt) => self.pre_walk_do_while_statement(stmt),
      Stmt::ForIn(stmt) => self.pre_walk_for_in_statement(stmt),
      Stmt::ForOf(stmt) => self.pre_walk_for_of_statement(stmt),
      Stmt::For(stmt) => self.pre_walk_for_statement(stmt),
      Stmt::Decl(decl) => match decl {
        Decl::Fn(decl) => self.pre_walk_function_declaration(decl),
        Decl::Var(decl) => self.pre_walk_variable_declaration(decl),
        Decl::Class(_) | Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::If(stmt) => self.pre_walk_if_statement(stmt),
      Stmt::Labeled(stmt) => self.pre_walk_labeled_statement(stmt),
      Stmt::Switch(stmt) => self.pre_walk_switch_statement(stmt),
      Stmt::Try(stmt) => self.pre_walk_try_statement(stmt),
      Stmt::While(stmt) => self.pre_walk_while_statement(stmt),
      Stmt::With(stmt) => self.pre_walk_with_statement(stmt),
      _ => (),
    }
  }

  fn pre_walk_with_statement(&mut self, stmt: &WithStmt) {
    self.pre_walk_statement(&stmt.body)
  }

  fn pre_walk_while_statement(&mut self, stmt: &WhileStmt) {
    self.pre_walk_statement(&stmt.body)
  }

  fn pre_walk_catch_clause(&mut self, cache_clause: &CatchClause) {
    // FIXME: webpack use `pre_walk_statement` here
    self.pre_walk_block_statement(&cache_clause.body);
  }

  fn pre_walk_try_statement(&mut self, stmt: &TryStmt) {
    // FIXME: webpack use `pre_walk_statement` here
    self.pre_walk_block_statement(&stmt.block);
    if let Some(handler) = &stmt.handler {
      self.pre_walk_catch_clause(handler)
    }
    if let Some(finalizer) = &stmt.finalizer {
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_block_statement(finalizer)
    }
  }

  fn pre_walk_switch_cases(&mut self, switch_cases: &Vec<SwitchCase>) {
    for switch_case in switch_cases {
      self.pre_walk_statements(&switch_case.cons)
    }
  }

  fn pre_walk_switch_statement(&mut self, stmt: &SwitchStmt) {
    self.pre_walk_switch_cases(&stmt.cases)
  }

  fn pre_walk_labeled_statement(&mut self, stmt: &LabeledStmt) {
    self.pre_walk_statement(&stmt.body);
  }

  fn pre_walk_if_statement(&mut self, stmt: &IfStmt) {
    self.pre_walk_statement(&stmt.cons);
    if let Some(alter) = &stmt.alt {
      self.pre_walk_statement(alter);
    }
  }

  fn pre_walk_function_declaration(&mut self, decl: &FnDecl) {
    self.define_variable(decl.ident.sym.to_string());
  }

  fn pre_walk_for_statement(&mut self, stmt: &ForStmt) {
    if let Some(decl) = stmt.init.as_ref().and_then(|init| init.as_var_decl()) {
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_variable_declaration(decl)
    }
    self.pre_walk_statement(&stmt.body);
  }

  fn pre_walk_for_of_statement(&mut self, stmt: &ForOfStmt) {
    if stmt.is_await && matches!(self.top_level_scope, super::TopLevelScope::Top) {
      self
        .plugin_drive
        .clone()
        .top_level_for_of_await_stmt(self, stmt);
    }
    if let Some(left) = stmt.left.as_var_decl() {
      self.pre_walk_variable_declaration(left)
    }
    self.pre_walk_statement(&stmt.body)
  }

  pub(super) fn pre_walk_block_statement(&mut self, stmt: &BlockStmt) {
    self.pre_walk_statements(&stmt.stmts);
  }

  fn pre_walk_do_while_statement(&mut self, stmt: &DoWhileStmt) {
    self.pre_walk_statement(&stmt.body);
  }

  fn pre_walk_for_in_statement(&mut self, stmt: &ForInStmt) {
    if let Some(decl) = stmt.left.as_var_decl() {
      self.pre_walk_variable_declaration(decl);
    }
    self.pre_walk_statement(&stmt.body);
  }

  fn pre_walk_variable_declaration(&mut self, decl: &VarDecl) {
    if decl.kind == VarDeclKind::Var {
      self._pre_walk_variable_declaration(decl)
    }
  }

  pub(super) fn _pre_walk_variable_declaration(&mut self, decl: &VarDecl) {
    for declarator in &decl.decls {
      self.pre_walk_variable_declarator(declarator);
      // TODO: hooks.pre_declarator
      self.enter_pattern(Cow::Borrowed(&declarator.name), |this, ident| {
        this.define_variable(ident.sym.to_string());
      });
    }
  }

  fn pre_walk_variable_declarator(&mut self, declarator: &VarDeclarator) {
    let Some(init) = declarator.init.as_ref() else {
      return;
    };
    let Some(obj_pat) = declarator.name.as_object() else {
      return;
    };
    let keys = self._pre_walk_object_pattern(obj_pat);
    if keys.is_none() {
      return;
    }
    if let Some(assign) = init.as_assign() {
      self.pre_walk_assignment_expression(assign);
    }
  }

  fn _pre_walk_object_pattern(&mut self, obj_pat: &ObjectPat) -> Option<FxHashSet<String>> {
    let mut keys = FxHashSet::default();
    for prop in &obj_pat.props {
      match prop {
        ObjectPatProp::KeyValue(prop) => {
          let name = eval::eval_prop_name(&prop.key);
          if let Some(id) = name.and_then(|id| id.as_string()) {
            keys.insert(id);
          }
        }
        ObjectPatProp::Assign(prop) => {
          keys.insert(prop.key.sym.to_string());
        }
        ObjectPatProp::Rest(_) => return None,
      };
    }
    Some(keys)
  }

  pub(super) fn pre_walk_assignment_expression(&mut self, assign: &AssignExpr) {
    // Check if the assignment left-hand side is an object pattern
    if let Some(obj_pat) = assign.left.as_pat().and_then(|pat| pat.as_object()) {
      // Extract keys from the object pattern
      if let Some(keys) = self._pre_walk_object_pattern(obj_pat) {
        // Handle nested assignments
        self.handle_destructuring_assignment(&assign, keys);

        // Map right-hand side expressions to assignments when nested
        if let Some(expr) = assign.right.as_assign() {
          self.pre_walk_assignment_expression(expr);
        }
      }
    }
  }

  fn handle_destructuring_assignment(&mut self, assign: &AssignExpr, keys: FxHashSet<String>) {
    let mut keys = keys;
    // Check multiple assignments
    if let Some(destructuring_assignment_properties) = &mut self.destructuring_assignment_properties
    {
      if let Some(set) = destructuring_assignment_properties.remove(&assign) {
        for id in set.iter() {
          keys.insert(id.clone());
        }
      }
    }

    // Associate the keys with the right-hand side expression
    let right_expr = match &*assign.right {
      Expr::Assign(assign_expr) => Some(assign_expr.clone()),
      Expr::Await(await_expr) => match &*await_expr.arg {
        Expr::Assign(assign_expr) => Some(assign_expr.clone()),
        _ => None,
      },
      _ => None,
    };
    dbg!(&right_expr);
    if let Some(destructuring_assignment_properties) = &mut self.destructuring_assignment_properties
    {
      if let Some(right_expr) = right_expr {
        destructuring_assignment_properties.insert(right_expr, keys.clone());
      }
    }
  }
}
