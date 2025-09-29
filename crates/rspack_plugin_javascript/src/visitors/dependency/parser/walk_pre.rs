use std::borrow::Cow;

use swc_core::{
  common::Spanned,
  ecma::ast::{
    ArrayPat, AssignExpr, BlockStmt, CatchClause, Decl, DoWhileStmt, ForHead, ForInStmt, ForOfStmt,
    ForStmt, IfStmt, LabeledStmt, ModuleDecl, ModuleItem, ObjectPat, ObjectPatProp, Pat, Stmt,
    SwitchCase, SwitchStmt, TryStmt, VarDeclarator, WhileStmt, WithStmt,
  },
};

use super::{
  DestructuringAssignmentProperty, JavascriptParser,
  estree::{MaybeNamedFunctionDecl, Statement},
};
use crate::{
  parser_plugin::JavascriptParserPlugin,
  utils::eval,
  visitors::{DestructuringAssignmentProperties, VariableDeclaration, VariableDeclarationKind},
};

impl JavascriptParser<'_> {
  pub fn pre_walk_module_items(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.pre_walk_module_item(statement);
    }
  }

  pub fn pre_walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.pre_walk_statement(statement.into())
    }
  }

  fn pre_walk_module_item(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        match decl {
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
          _ => {
            self.is_esm = true;
          }
        };
      }
      ModuleItem::Stmt(stmt) => self.pre_walk_statement(stmt.into()),
    }
  }

  pub fn pre_walk_statement(&mut self, statement: Statement) {
    self.enter_statement(
      &statement,
      |parser, _| {
        parser
          .plugin_drive
          .clone()
          .pre_statement(parser, statement)
          .unwrap_or_default()
      },
      |parser, _| {
        match statement {
          Statement::Block(stmt) => parser.pre_walk_block_statement(stmt),
          Statement::DoWhile(stmt) => parser.pre_walk_do_while_statement(stmt),
          Statement::ForIn(stmt) => parser.pre_walk_for_in_statement(stmt),
          Statement::ForOf(stmt) => parser.pre_walk_for_of_statement(stmt),
          Statement::For(stmt) => parser.pre_walk_for_statement(stmt),
          Statement::Fn(stmt) => parser.pre_walk_function_declaration(stmt),
          Statement::Var(stmt) => parser.pre_walk_variable_declaration(stmt),
          Statement::If(stmt) => parser.pre_walk_if_statement(stmt),
          Statement::Labeled(stmt) => parser.pre_walk_labeled_statement(stmt),
          Statement::Switch(stmt) => parser.pre_walk_switch_statement(stmt),
          Statement::Try(stmt) => parser.pre_walk_try_statement(stmt),
          Statement::While(stmt) => parser.pre_walk_while_statement(stmt),
          Statement::With(stmt) => parser.pre_walk_with_statement(stmt),
          _ => (),
        };
      },
    );
  }

  pub fn pre_walk_declaration(&mut self, decl: &Decl) {
    match decl {
      Decl::Fn(decl) => self.pre_walk_function_declaration(decl.into()),
      Decl::Var(decl) => self.pre_walk_variable_declaration(VariableDeclaration::VarDecl(decl)),
      Decl::Using(decl) => self.pre_walk_variable_declaration(VariableDeclaration::UsingDecl(decl)),
      Decl::Class(_) => (),
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!()
      }
    }
  }

  fn pre_walk_with_statement(&mut self, stmt: &WithStmt) {
    self.pre_walk_statement(stmt.body.as_ref().into())
  }

  fn pre_walk_while_statement(&mut self, stmt: &WhileStmt) {
    self.pre_walk_statement(stmt.body.as_ref().into())
  }

  fn pre_walk_catch_clause(&mut self, cache_clause: &CatchClause) {
    self.pre_walk_statement(Statement::Block(&cache_clause.body));
  }

  fn pre_walk_try_statement(&mut self, stmt: &TryStmt) {
    self.pre_walk_statement(Statement::Block(&stmt.block));
    if let Some(handler) = &stmt.handler {
      self.pre_walk_catch_clause(handler)
    }
    if let Some(finalizer) = &stmt.finalizer {
      self.pre_walk_statement(Statement::Block(finalizer));
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
    self.pre_walk_statement(stmt.body.as_ref().into());
  }

  fn pre_walk_if_statement(&mut self, stmt: &IfStmt) {
    self.pre_walk_statement(stmt.cons.as_ref().into());
    if let Some(alter) = &stmt.alt {
      self.pre_walk_statement(alter.as_ref().into());
    }
  }

  pub fn pre_walk_function_declaration(&mut self, decl: MaybeNamedFunctionDecl) {
    if let Some(ident) = decl.ident() {
      self.define_variable(ident.sym.clone());
    }
  }

  fn pre_walk_for_statement(&mut self, stmt: &ForStmt) {
    if let Some(decl) = stmt.init.as_ref().and_then(|init| init.as_var_decl()) {
      self.pre_walk_statement(Statement::Var(VariableDeclaration::VarDecl(decl)))
    }
    self.pre_walk_statement(stmt.body.as_ref().into());
  }

  fn pre_walk_for_head(&mut self, head: &ForHead) {
    match head {
      ForHead::VarDecl(decl) => {
        self.pre_walk_variable_declaration(VariableDeclaration::VarDecl(decl))
      }
      ForHead::UsingDecl(decl) => {
        self.pre_walk_variable_declaration(VariableDeclaration::UsingDecl(decl))
      }
      ForHead::Pat(_) => {}
    }
  }

  fn pre_walk_for_of_statement(&mut self, stmt: &ForOfStmt) {
    if stmt.is_await && self.is_top_level_scope() {
      self
        .plugin_drive
        .clone()
        .top_level_for_of_await_stmt(self, stmt);
    }
    self.pre_walk_for_head(&stmt.left);
    self.pre_walk_statement(stmt.body.as_ref().into())
  }

  pub(super) fn pre_walk_block_statement(&mut self, stmt: &BlockStmt) {
    self.pre_walk_statements(&stmt.stmts);
  }

  fn pre_walk_do_while_statement(&mut self, stmt: &DoWhileStmt) {
    self.pre_walk_statement(stmt.body.as_ref().into());
  }

  fn pre_walk_for_in_statement(&mut self, stmt: &ForInStmt) {
    self.pre_walk_for_head(&stmt.left);
    self.pre_walk_statement(stmt.body.as_ref().into());
  }

  fn pre_walk_variable_declaration(&mut self, decl: VariableDeclaration<'_>) {
    if decl.kind() == VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(decl)
    }
  }

  pub(super) fn _pre_walk_variable_declaration(&mut self, decl: VariableDeclaration<'_>) {
    for declarator in decl.declarators() {
      self.pre_walk_variable_declarator(declarator);
      if !self
        .plugin_drive
        .clone()
        .pre_declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.enter_pattern(Cow::Borrowed(&declarator.name), |this, ident| {
          this.define_variable(ident.sym.clone());
        });
      }
    }
  }

  pub(crate) fn collect_destructuring_assignment_properties(
    &mut self,
    pattern: &Pat,
  ) -> Option<DestructuringAssignmentProperties> {
    if let Some(obj_pat) = pattern.as_object()
      && let Some(properties) =
        self.collect_destructuring_assignment_properties_from_object_pattern(obj_pat)
    {
      return Some(properties);
    }
    if let Some(arr_pat) = pattern.as_array()
      && let Some(properties) =
        self.collect_destructuring_assignment_properties_from_array_pattern(arr_pat)
    {
      return Some(properties);
    }
    None
  }

  pub(crate) fn collect_destructuring_assignment_properties_from_object_pattern(
    &mut self,
    obj_pat: &ObjectPat,
  ) -> Option<DestructuringAssignmentProperties> {
    let mut keys = DestructuringAssignmentProperties::default();
    for prop in &obj_pat.props {
      match prop {
        ObjectPatProp::KeyValue(prop) => {
          if let Some(ident_key) = prop.key.as_ident() {
            keys.insert(DestructuringAssignmentProperty {
              id: ident_key.sym.clone(),
              range: prop.key.span().into(),
              pattern: self.collect_destructuring_assignment_properties(&prop.value),
              shorthand: false,
            });
          } else {
            let name = eval::eval_prop_name(self, &prop.key);
            if let Some(id) = name.as_string() {
              keys.insert(DestructuringAssignmentProperty {
                id: id.into(),
                range: prop.key.span().into(),
                pattern: self.collect_destructuring_assignment_properties(&prop.value),
                shorthand: false,
              });
            } else {
              return None;
            }
          }
        }
        ObjectPatProp::Assign(prop) => {
          keys.insert(DestructuringAssignmentProperty {
            id: prop.key.sym.clone(),
            range: prop.key.span().into(),
            pattern: None,
            shorthand: true,
          });
        }
        ObjectPatProp::Rest(_) => return None,
      };
    }
    Some(keys)
  }

  pub(crate) fn collect_destructuring_assignment_properties_from_array_pattern(
    &mut self,
    arr_pat: &ArrayPat,
  ) -> Option<DestructuringAssignmentProperties> {
    let mut keys = DestructuringAssignmentProperties::default();
    for (i, ele) in arr_pat.elems.iter().enumerate() {
      let Some(ele) = ele else {
        continue;
      };
      if ele.is_rest() {
        return None;
      }
      let mut buf = rspack_util::itoa::Buffer::new();
      let i = buf.format(i);
      keys.insert(DestructuringAssignmentProperty {
        id: i.into(),
        range: ele.span().into(),
        pattern: self.collect_destructuring_assignment_properties(ele),
        shorthand: false,
      });
    }
    Some(keys)
  }

  fn pre_walk_variable_declarator(&mut self, declarator: &VarDeclarator) {
    let Some(init) = declarator.init.as_ref() else {
      return;
    };
    let Some(obj_pat) = declarator.name.as_object() else {
      return;
    };
    self.enter_destructuring_assignment(obj_pat, init);
  }

  pub(crate) fn pre_walk_assignment_expression(&mut self, assign: &AssignExpr) {
    if let Some(pat) = assign.left.as_pat()
      && let Some(obj_pat) = pat.as_object()
    {
      self.enter_destructuring_assignment(obj_pat, &assign.right);
    }
  }
}
