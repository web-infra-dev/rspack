use swc_experimental_ecma_ast::{
  ArrayPat, AssignExpr, BlockStmt, CatchClause, DoWhileStmt, ForHead, ForInStmt, ForOfStmt,
  ForStmt, IfStmt, LabeledStmt, ModuleItem, ObjectPat, ObjectPatProp, Pat, Spanned, Stmt,
  SwitchCase, SwitchStmt, TryStmt, TypedSubRange, VarDeclarator, WhileStmt, WithStmt,
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
  pub fn pre_walk_module_items(&mut self, statements: TypedSubRange<ModuleItem>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.pre_walk_module_item(statement);
    }
  }

  pub fn pre_walk_statements(&mut self, statements: TypedSubRange<Stmt>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.pre_walk_statement(Statement::from_stmt(statement, &self.ast))
    }
  }

  fn pre_walk_module_item(&mut self, statement: ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(_) => {
        self.is_esm = true;
      }
      ModuleItem::Stmt(stmt) => self.pre_walk_statement(Statement::from_stmt(stmt, &self.ast)),
    }
  }

  pub fn pre_walk_statement(&mut self, statement: Statement) {
    self.enter_statement(
      statement,
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

  fn pre_walk_with_statement(&mut self, stmt: WithStmt) {
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast))
  }

  fn pre_walk_while_statement(&mut self, stmt: WhileStmt) {
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast))
  }

  fn pre_walk_catch_clause(&mut self, cache_clause: CatchClause) {
    self.pre_walk_statement(Statement::Block(cache_clause.body(&self.ast)));
  }

  fn pre_walk_try_statement(&mut self, stmt: TryStmt) {
    self.pre_walk_statement(Statement::Block(stmt.block(&self.ast)));
    if let Some(handler) = stmt.handler(&self.ast) {
      self.pre_walk_catch_clause(handler)
    }
    if let Some(finalizer) = stmt.finalizer(&self.ast) {
      self.pre_walk_statement(Statement::Block(finalizer));
    }
  }

  fn pre_walk_switch_cases(&mut self, switch_cases: TypedSubRange<SwitchCase>) {
    for switch_case in switch_cases.iter() {
      let switch_case = self.ast.get_node_in_sub_range(switch_case);
      self.pre_walk_statements(switch_case.cons(&self.ast))
    }
  }

  fn pre_walk_switch_statement(&mut self, stmt: SwitchStmt) {
    self.pre_walk_switch_cases(stmt.cases(&self.ast))
  }

  fn pre_walk_labeled_statement(&mut self, stmt: LabeledStmt) {
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast));
  }

  fn pre_walk_if_statement(&mut self, stmt: IfStmt) {
    self.pre_walk_statement(Statement::from_stmt(stmt.cons(&self.ast), &self.ast));
    if let Some(alter) = stmt.alt(&self.ast) {
      self.pre_walk_statement(Statement::from_stmt(alter, &self.ast));
    }
  }

  pub fn pre_walk_function_declaration(&mut self, decl: MaybeNamedFunctionDecl) {
    if let Some(ident) = decl.ident() {
      self.define_variable(self.ast.get_atom(ident.sym(&self.ast)));
    }
  }

  fn pre_walk_for_statement(&mut self, stmt: ForStmt) {
    if let Some(decl) = stmt.init(&self.ast).and_then(|init| init.as_var_decl()) {
      self.pre_walk_statement(Statement::Var(VariableDeclaration::VarDecl(decl)))
    }
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast));
  }

  fn pre_walk_for_head(&mut self, head: ForHead) {
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

  fn pre_walk_for_of_statement(&mut self, stmt: ForOfStmt) {
    if stmt.is_await(&self.ast) && self.is_top_level_scope() {
      self
        .plugin_drive
        .clone()
        .top_level_for_of_await_stmt(self, stmt);
    }
    self.pre_walk_for_head(stmt.left(&self.ast));
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast))
  }

  pub(super) fn pre_walk_block_statement(&mut self, stmt: BlockStmt) {
    self.pre_walk_statements(stmt.stmts(&self.ast));
  }

  fn pre_walk_do_while_statement(&mut self, stmt: DoWhileStmt) {
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast));
  }

  fn pre_walk_for_in_statement(&mut self, stmt: ForInStmt) {
    self.pre_walk_for_head(stmt.left(&self.ast));
    self.pre_walk_statement(Statement::from_stmt(stmt.body(&self.ast), &self.ast));
  }

  fn pre_walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    if decl.kind(&self.ast) == VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(decl)
    }
  }

  pub(super) fn _pre_walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    for declarator in decl.declarators(&self.ast).iter() {
      let declarator = self.ast.get_node_in_sub_range(declarator);
      self.pre_walk_variable_declarator(declarator);
      if !self
        .plugin_drive
        .clone()
        .pre_declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.enter_pattern(declarator.name(&self.ast), |this, ident| {
          this.define_variable(this.ast.get_atom(ident.sym(&this.ast)));
        });
      }
    }
  }

  pub(crate) fn collect_destructuring_assignment_properties(
    &mut self,
    pattern: Pat,
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
    obj_pat: ObjectPat,
  ) -> Option<DestructuringAssignmentProperties> {
    let mut keys = DestructuringAssignmentProperties::default();
    for prop in obj_pat.props(&self.ast).iter() {
      let prop = self.ast.get_node_in_sub_range(prop);
      match prop {
        ObjectPatProp::KeyValue(prop) => {
          if let Some(ident_key) = prop.key(&self.ast).as_ident() {
            keys.insert(DestructuringAssignmentProperty {
              id: self.ast.get_atom(ident_key.sym(&self.ast)),
              range: prop.key(&self.ast).span(&self.ast).into(),
              pattern: self.collect_destructuring_assignment_properties(prop.value(&self.ast)),
              shorthand: false,
            });
          } else {
            let name = eval::eval_prop_name(self, prop.key(&self.ast));
            if let Some(id) = name.as_string() {
              keys.insert(DestructuringAssignmentProperty {
                id: id.into(),
                range: prop.key(&self.ast).span(&self.ast).into(),
                pattern: self.collect_destructuring_assignment_properties(prop.value(&self.ast)),
                shorthand: false,
              });
            } else {
              return None;
            }
          }
        }
        ObjectPatProp::Assign(prop) => {
          keys.insert(DestructuringAssignmentProperty {
            id: self
              .ast
              .get_atom(prop.key(&self.ast).id(&self.ast).sym(&self.ast)),
            range: prop.key(&self.ast).span(&self.ast).into(),
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
    arr_pat: ArrayPat,
  ) -> Option<DestructuringAssignmentProperties> {
    let mut keys = DestructuringAssignmentProperties::default();
    for (i, ele) in arr_pat.elems(&self.ast).iter().enumerate() {
      let Some(ele) = self.ast.get_node_in_sub_range(ele) else {
        continue;
      };
      if ele.is_rest() {
        return None;
      }
      let mut buf = rspack_util::itoa::Buffer::new();
      let i = buf.format(i);
      keys.insert(DestructuringAssignmentProperty {
        id: i.into(),
        range: ele.span(&self.ast).into(),
        pattern: self.collect_destructuring_assignment_properties(ele),
        shorthand: false,
      });
    }
    Some(keys)
  }

  fn pre_walk_variable_declarator(&mut self, declarator: VarDeclarator) {
    let Some(init) = declarator.init(&self.ast) else {
      return;
    };
    let Some(obj_pat) = declarator.name(&self.ast).as_object() else {
      return;
    };
    self.enter_destructuring_assignment(obj_pat, init);
  }

  pub(crate) fn pre_walk_assignment_expression(&mut self, assign: AssignExpr) {
    if let Some(pat) = assign.left(&self.ast).as_pat()
      && let Some(obj_pat) = pat.as_object()
    {
      self.enter_destructuring_assignment(obj_pat, assign.right(&self.ast));
    }
  }
}
