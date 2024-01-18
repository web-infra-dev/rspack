use std::borrow::Cow;

use rustc_hash::FxHashSet;
use swc_core::ecma::ast::FnDecl;
use swc_core::ecma::ast::{AssignExpr, BlockStmt, CatchClause, Decl, DoWhileStmt, ExprStmt};
use swc_core::ecma::ast::{ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt, WithStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, ObjectPat, ObjectPatProp, Stmt, WhileStmt};
use swc_core::ecma::ast::{SwitchCase, SwitchStmt, TryStmt, VarDecl, VarDeclKind, VarDeclarator};

use super::JavascriptParser;
use crate::parser_plugin::JavaScriptParserPluginDrive;
use crate::utils::eval;

impl<'ast, 'parser> JavascriptParser<'parser> {
  pub fn pre_walk_module_declarations(
    &mut self,
    statements: &'ast Vec<ModuleItem>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.pre_walk_module_declaration(statement, plugin_drive);
    }
  }

  pub fn pre_walk_statements(
    &mut self,
    statements: &'ast Vec<Stmt>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.pre_walk_statement(statement, plugin_drive)
    }
  }

  fn pre_walk_module_declaration(
    &mut self,
    statement: &'ast ModuleItem,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match statement {
      ModuleItem::ModuleDecl(decl) => match decl {
        ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        _ => (),
      },
      ModuleItem::Stmt(stmt) => self.pre_walk_statement(stmt, plugin_drive),
    }
  }

  pub fn pre_walk_statement(
    &mut self,
    statement: &'ast Stmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: hooks.preStatement call
    match statement {
      Stmt::Block(stmt) => self.pre_walk_block_statement(stmt, plugin_drive),
      Stmt::DoWhile(stmt) => self.pre_walk_do_while_statement(stmt, plugin_drive),
      Stmt::ForIn(stmt) => self.pre_walk_for_in_statement(stmt, plugin_drive),
      Stmt::ForOf(stmt) => self.pre_walk_for_of_statement(stmt, plugin_drive),
      Stmt::For(stmt) => self.pre_walk_for_statement(stmt, plugin_drive),
      Stmt::Decl(decl) => match decl {
        Decl::Fn(decl) => self.pre_walk_function_declaration(decl, plugin_drive),
        Decl::Var(decl) => self.pre_walk_variable_declaration(decl, plugin_drive),
        Decl::Class(_) | Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::If(stmt) => self.pre_walk_if_statement(stmt, plugin_drive),
      Stmt::Labeled(stmt) => self.pre_walk_labeled_statement(stmt, plugin_drive),
      Stmt::Switch(stmt) => self.pre_walk_switch_statement(stmt, plugin_drive),
      Stmt::Try(stmt) => self.pre_walk_try_statement(stmt, plugin_drive),
      Stmt::While(stmt) => self.pre_walk_while_statement(stmt, plugin_drive),
      Stmt::With(stmt) => self.pre_walk_with_statement(stmt, plugin_drive),
      _ => (),
    }
  }

  fn pre_walk_with_statement(
    &mut self,
    stmt: &'ast WithStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statement(&stmt.body, plugin_drive)
  }

  fn pre_walk_while_statement(
    &mut self,
    stmt: &'ast WhileStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statement(&stmt.body, plugin_drive)
  }

  fn pre_walk_catch_clause(
    &mut self,
    cache_clause: &'ast CatchClause,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // FIXME: webpack use `pre_walk_statement` here
    self.pre_walk_block_statement(&cache_clause.body, plugin_drive);
  }

  fn pre_walk_try_statement(
    &mut self,
    stmt: &'ast TryStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // FIXME: webpack use `pre_walk_statement` here
    self.pre_walk_block_statement(&stmt.block, plugin_drive);
    if let Some(handler) = &stmt.handler {
      self.pre_walk_catch_clause(handler, plugin_drive)
    }
    if let Some(finalizer) = &stmt.finalizer {
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_block_statement(finalizer, plugin_drive)
    }
  }

  fn pre_walk_switch_cases(
    &mut self,
    switch_cases: &'ast Vec<SwitchCase>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for switch_case in switch_cases {
      self.pre_walk_statements(&switch_case.cons, plugin_drive)
    }
  }

  fn pre_walk_switch_statement(
    &mut self,
    stmt: &'ast SwitchStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_switch_cases(&stmt.cases, plugin_drive);
  }

  fn pre_walk_labeled_statement(
    &mut self,
    stmt: &'ast LabeledStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statement(&stmt.body, plugin_drive);
  }

  fn pre_walk_if_statement(
    &mut self,
    stmt: &'ast IfStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statement(&stmt.cons, plugin_drive);
    if let Some(alter) = &stmt.alt {
      self.pre_walk_statement(alter, plugin_drive);
    }
  }

  fn pre_walk_function_declaration(
    &mut self,
    decl: &FnDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.define_variable(decl.ident.sym.as_str());
  }

  fn pre_walk_for_statement(
    &mut self,
    stmt: &'ast ForStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(decl) = stmt.init.as_ref().and_then(|init| init.as_var_decl()) {
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_variable_declaration(decl, plugin_drive)
    }
    self.pre_walk_statement(&stmt.body, plugin_drive);
  }

  fn pre_walk_for_of_statement(
    &mut self,
    stmt: &'ast ForOfStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: hooks.topLevelAwait call
    if let Some(left) = stmt.left.as_var_decl() {
      self.pre_walk_variable_declaration(left, plugin_drive)
    }
    self.pre_walk_statement(&stmt.body, plugin_drive)
  }

  pub(super) fn pre_walk_block_statement(
    &mut self,
    stmt: &'ast BlockStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statements(&stmt.stmts, plugin_drive);
  }

  fn pre_walk_do_while_statement(
    &mut self,
    stmt: &'ast DoWhileStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.pre_walk_statement(&stmt.body, plugin_drive);
  }

  fn pre_walk_for_in_statement(
    &mut self,
    stmt: &'ast ForInStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(decl) = stmt.left.as_var_decl() {
      self.pre_walk_variable_declaration(decl, plugin_drive);
    }
    self.pre_walk_statement(&stmt.body, plugin_drive);
  }

  fn pre_walk_variable_declaration(
    &mut self,
    decl: &VarDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if decl.kind == VarDeclKind::Var {
      self._pre_walk_variable_declaration(decl, plugin_drive);
    }
  }

  pub(super) fn _pre_walk_variable_declaration(
    &mut self,
    decl: &VarDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for declarator in &decl.decls {
      self.pre_walk_variable_declarator(declarator, plugin_drive);
      // TODO: hooks.pre_declarator
      self.enter_pattern(&declarator.name, |this, ident| {
        this.define_variable(ident.sym.as_str());
      });
    }
  }

  fn _pre_walk_object_pattern(&mut self, obj_pat: &'ast ObjectPat) -> Option<FxHashSet<String>> {
    let mut keys = FxHashSet::default();
    for prop in &obj_pat.props {
      match prop {
        ObjectPatProp::KeyValue(prop) => {
          let name = eval::eval_prop_name(&prop.key);
          if let Some(id) = name
            && let Some(id) = id.as_string()
          {
            // FIXME: can we delete `to_string?`
            keys.insert(id.to_string());
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

  fn pre_walk_variable_declarator(
    &mut self,
    declarator: &VarDeclarator,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
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
      self.pre_walk_assignment_expression(assign, plugin_drive);
    }
  }

  pub(super) fn pre_walk_assignment_expression(
    &mut self,
    assign: &AssignExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    let Some(left) = assign.left.as_pat().and_then(|pat| pat.as_object()) else {
      return;
    };
    let keys = self._pre_walk_object_pattern(left);
    if keys.is_none() {
      return;
    }
    if let Some(right) = assign.right.as_assign() {
      self.pre_walk_assignment_expression(right, _plugin_drive)
    }
  }
}
