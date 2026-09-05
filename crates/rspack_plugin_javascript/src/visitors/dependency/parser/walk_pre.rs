use rspack_util::SpanExt;
use swc_next_ecma_ast::*;

use super::{
  DestructuringAssignmentProperty, JavascriptParser, PatRef,
  estree::{MaybeNamedFunctionDecl, Statement},
};
use crate::{
  Atom,
  parser_plugin::JavascriptParserPlugin,
  utils::eval::{BasicEvaluatedExpression, parse_bigint_literal},
  visitors::{DestructuringAssignmentProperties, VariableDeclaration, VariableDeclarationKind},
};

fn eval_property_key<'parser>(
  parser: &mut JavascriptParser<'parser>,
  key: PropertyKey,
) -> BasicEvaluatedExpression<'parser> {
  let ast = parser.ast.ast;
  let span = key.span(ast);
  let mut evaluated = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  match ast.property_key_data(key) {
    PropertyKeyData::StringLiteral(string) => {
      evaluated.set_string(
        ast
          .get_wtf8(string.value(ast))
          .to_string_lossy()
          .into_owned(),
      );
    }
    PropertyKeyData::NumericLiteral(number) => evaluated.set_number(number.value(ast)),
    PropertyKeyData::BigIntLiteral(bigint) => {
      if let Some(value) = parse_bigint_literal(ast.get_utf8(bigint.raw(ast))) {
        evaluated.set_bigint(value);
      }
    }
    PropertyKeyData::IdentifierName(identifier) => {
      evaluated.set_string(ast.get_utf8(identifier.name(ast)).to_string());
    }
    PropertyKeyData::PrivateIdentifier(identifier) => {
      evaluated.set_string(ast.get_utf8(identifier.name(ast)).to_string());
    }
    PropertyKeyData::Expr(expression) => return parser.evaluate_expression(expression),
  }
  evaluated
}

fn skip_js_trivia(source: &[u8], mut offset: usize, end: usize) -> usize {
  while offset < end {
    if source[offset].is_ascii_whitespace() {
      offset += 1;
      continue;
    }
    if offset + 1 < end && source[offset] == b'/' && source[offset + 1] == b'/' {
      offset += 2;
      while offset < end && !matches!(source[offset], b'\n' | b'\r') {
        offset += 1;
      }
      continue;
    }
    if offset + 1 < end && source[offset] == b'/' && source[offset + 1] == b'*' {
      offset += 2;
      while offset + 1 < end && !(source[offset] == b'*' && source[offset + 1] == b'/') {
        offset += 1;
      }
      offset = (offset + 2).min(end);
      continue;
    }
    break;
  }
  offset
}

impl JavascriptParser<'_> {
  pub fn pre_walk_module_items(&mut self, statements: &[Stmt]) {
    for &statement in statements {
      self.pre_walk_module_item(statement);
    }
  }

  pub fn pre_walk_statements(&mut self, statements: &[Stmt]) {
    for &statement in statements {
      self.pre_walk_statement(Statement::from_stmt(self.ast.ast, statement));
    }
  }

  fn pre_walk_module_item(&mut self, statement: Stmt) {
    match self.ast.ast.stmt_data(statement) {
      StmtData::ImportDeclaration(_)
      | StmtData::ExportNamedDeclaration(_)
      | StmtData::ExportDefaultDeclaration(_)
      | StmtData::ExportAllDeclaration(_) => self.is_esm = true,
      _ => self.pre_walk_statement(Statement::from_stmt(self.ast.ast, statement)),
    }
  }

  pub fn pre_walk_statement(&mut self, statement: Statement) {
    let drive = self.plugin_drive.clone();
    self.enter_statement(
      statement.span(self.ast.ast),
      statement,
      |parser, node| drive.pre_statement(parser, node).unwrap_or_default(),
      |parser, node| match node {
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
      },
    );
  }

  fn pre_walk_with_statement(&mut self, stmt: WithStatement) {
    self.pre_walk_statement(Statement::from_stmt(self.ast.ast, stmt.body(self.ast.ast)));
  }

  fn pre_walk_while_statement(&mut self, stmt: WhileStatement) {
    self.pre_walk_statement(Statement::from_stmt(self.ast.ast, stmt.body(self.ast.ast)));
  }

  fn pre_walk_catch_clause(&mut self, clause: CatchClause) {
    self.pre_walk_statement(Statement::Block(clause.body(self.ast.ast)));
  }

  fn pre_walk_try_statement(&mut self, stmt: TryStatement) {
    let ast = self.ast.ast;
    self.pre_walk_statement(Statement::Block(stmt.block(ast)));
    if let Some(handler) = stmt.handler(ast) {
      self.pre_walk_catch_clause(handler);
    }
    if let Some(finalizer) = stmt.finalizer(ast) {
      self.pre_walk_statement(Statement::Block(finalizer));
    }
  }

  fn pre_walk_switch_statement(&mut self, stmt: SwitchStatement) {
    let ast = self.ast.ast;
    let cases = stmt
      .cases(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .collect::<Vec<_>>();
    for case in cases {
      let statements = case
        .consequent(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id))
        .collect::<Vec<_>>();
      self.pre_walk_statements(&statements);
    }
  }

  fn pre_walk_labeled_statement(&mut self, stmt: LabeledStatement) {
    self.pre_walk_statement(Statement::from_stmt(self.ast.ast, stmt.body(self.ast.ast)));
  }

  fn pre_walk_if_statement(&mut self, stmt: IfStatement) {
    let ast = self.ast.ast;
    self.pre_walk_statement(Statement::from_stmt(ast, stmt.consequent(ast)));
    if let Some(alternate) = stmt.alternate(ast) {
      self.pre_walk_statement(Statement::from_stmt(ast, alternate));
    }
  }

  pub fn pre_walk_function_declaration(&mut self, decl: MaybeNamedFunctionDecl) {
    if let Some(identifier) = decl.ident(self.ast.ast) {
      self.define_variable(Atom::from(
        self.ast.ast.get_utf8(identifier.name(self.ast.ast)),
      ));
    }
  }

  fn pre_walk_for_statement(&mut self, stmt: ForStatement) {
    let ast = self.ast.ast;
    if let Some(init) = stmt.init(ast)
      && let ForStatementInitData::VariableDeclaration(declaration) =
        ast.for_statement_init_data(init)
    {
      self.pre_walk_statement(Statement::Var(VariableDeclaration(declaration)));
    }
    self.pre_walk_statement(Statement::from_stmt(ast, stmt.body(ast)));
  }

  fn pre_walk_for_head(&mut self, head: ForStatementLeft) {
    if let ForStatementLeftData::VariableDeclaration(declaration) =
      self.ast.ast.for_statement_left_data(head)
    {
      self.pre_walk_variable_declaration(VariableDeclaration(declaration));
    }
  }

  fn pre_walk_for_of_statement(&mut self, stmt: ForOfStatement) {
    let ast = self.ast.ast;
    if stmt.r#await(ast) && self.is_top_level_scope() {
      self
        .plugin_drive
        .clone()
        .top_level_for_of_await_stmt(self, stmt);
    }
    self.pre_walk_for_head(stmt.left(ast));
    self.pre_walk_statement(Statement::from_stmt(ast, stmt.body(ast)));
  }

  pub(super) fn pre_walk_block_statement(&mut self, stmt: BlockStatement) {
    let statements = stmt
      .body(self.ast.ast)
      .iter()
      .map(|id| self.ast.ast.get_node_in_sub_range(id))
      .collect::<Vec<_>>();
    self.pre_walk_statements(&statements);
  }

  fn pre_walk_do_while_statement(&mut self, stmt: DoWhileStatement) {
    self.pre_walk_statement(Statement::from_stmt(self.ast.ast, stmt.body(self.ast.ast)));
  }

  fn pre_walk_for_in_statement(&mut self, stmt: ForInStatement) {
    let ast = self.ast.ast;
    self.pre_walk_for_head(stmt.left(ast));
    self.pre_walk_statement(Statement::from_stmt(ast, stmt.body(ast)));
  }

  fn pre_walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    if decl.kind(self.ast.ast) == VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(decl);
    }
  }

  pub(super) fn _pre_walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    let drive = self.plugin_drive.clone();
    let ast = self.ast.ast;
    for declarator in decl.declarators(ast) {
      self.pre_walk_variable_declarator(declarator);
      if !drive
        .pre_declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.enter_pattern(PatRef::Borrowed(declarator.id(ast)), |this, identifier| {
          this.define_variable(Atom::from(
            this.ast.ast.get_utf8(identifier.name(this.ast.ast)),
          ));
        });
      }
    }
  }

  pub(crate) fn collect_destructuring_assignment_properties(
    &mut self,
    pattern: BindingPattern,
  ) -> Option<DestructuringAssignmentProperties> {
    let ast = self.ast.ast;
    if let Some(object) = pattern.as_object_pattern(ast) {
      return self.collect_destructuring_assignment_properties_from_object_pattern(object);
    }
    if let Some(array) = pattern.as_array_pattern(ast) {
      return self.collect_destructuring_assignment_properties_from_array_pattern(array);
    }
    None
  }

  pub(crate) fn collect_destructuring_assignment_properties_from_object_pattern(
    &mut self,
    object: ObjectPattern,
  ) -> Option<DestructuringAssignmentProperties> {
    let ast = self.ast.ast;
    if object.rest(ast).is_some() {
      return None;
    }
    let mut keys = DestructuringAssignmentProperties::default();
    let properties = object
      .properties(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .collect::<Vec<_>>();
    for property in properties {
      let key = property.key(ast);
      let value = property.value(ast);
      let (id, shorthand) = if property.shorthand(ast) {
        let identifier = value.as_binding_identifier(ast).or_else(|| {
          value
            .as_assignment_pattern(ast)
            .and_then(|assignment| assignment.left(ast).as_binding_identifier(ast))
        })?;
        (Atom::from(ast.get_utf8(identifier.name(ast))), true)
      } else {
        let evaluated = eval_property_key(self, key);
        (Atom::from(evaluated.as_string()?), false)
      };
      let key_span = key.span(ast);
      let range = if property.computed(ast) {
        // SWC Next stores a computed PropertyKey span for the expression
        // inside `[...]`, while the legacy AST span included the brackets.
        // The export-mangling template replaces the key with an identifier,
        // so include the brackets to avoid emitting `[mangledName]` (a runtime
        // variable lookup) instead of `mangledName` (a property key).
        let property_span = property.span(ast);
        let source = self.source().as_bytes();
        let property_start = property_span.real_lo() as usize;
        let property_end = property_span.real_hi() as usize;
        let key_start = key_span.real_lo() as usize;
        let key_end = key_span.real_hi() as usize;
        let opening_bracket = skip_js_trivia(source, property_start, key_start);
        let closing_bracket = skip_js_trivia(source, key_end, property_end);
        let start = if source.get(opening_bracket) == Some(&b'[') {
          opening_bracket
        } else {
          key_start
        };
        let end = if source.get(closing_bracket) == Some(&b']') {
          closing_bracket + 1
        } else {
          key_end
        };
        (start as u32, end as u32).into()
      } else {
        key_span.into()
      };
      keys.insert(DestructuringAssignmentProperty {
        id,
        range,
        pattern: self.collect_destructuring_assignment_properties(value),
        shorthand,
      });
    }
    Some(keys)
  }

  pub(crate) fn collect_destructuring_assignment_properties_from_array_pattern(
    &mut self,
    array: ArrayPattern,
  ) -> Option<DestructuringAssignmentProperties> {
    let ast = self.ast.ast;
    if array.rest(ast).is_some() {
      return None;
    }
    let mut keys = DestructuringAssignmentProperties::default();
    let mut buffer = rspack_util::itoa::Buffer::new();
    for (index, element) in array
      .elements(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .enumerate()
    {
      let Some(element) = element else {
        continue;
      };
      keys.insert(DestructuringAssignmentProperty {
        id: buffer.format(index).into(),
        range: element.span(ast).into(),
        pattern: self.collect_destructuring_assignment_properties(element),
        shorthand: false,
      });
    }
    Some(keys)
  }

  fn pre_walk_variable_declarator(&mut self, declarator: VariableDeclarator) {
    let ast = self.ast.ast;
    let Some(init) = declarator.init(ast) else {
      return;
    };
    let Some(object) = declarator.id(ast).as_object_pattern(ast) else {
      return;
    };
    self.enter_destructuring_assignment(object, init);
  }

  pub(crate) fn pre_walk_assignment_expression(&mut self, assign: AssignmentExpression) {
    let ast = self.ast.ast;
    if let Some(object) = assign.left(ast).as_object_assignment_target(ast) {
      self.enter_destructuring_assignment(object, assign.right(ast));
    }
  }
}
