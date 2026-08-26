use smallvec::SmallVec;
use swc_next_ecma_ast::*;

use super::{
  AllowedMemberTypes, CallHooksName, ExpressionExpressionInfo, JavascriptParser,
  MemberExpressionInfo, MemberRanges, OptionalMembers, PatRef, RootName, ScopeTerminated,
  TopLevelScope,
  estree::{
    ClassDeclOrExpr, ExportDefaultDeclaration, MaybeNamedClassDecl, MaybeNamedFunctionDecl,
    Statement,
  },
  object_and_members_to_name,
};
use crate::{
  Atom,
  dependency::DependencyBranchGuard,
  parser_plugin::{
    CREATE_REQUIRE_EVALUATED_TAG, CREATE_REQUIRE_SPECIFIER_TAG, CREATED_REQUIRE_IDENTIFIER_TAG,
    CreatedRequireTagData, JavascriptParserPlugin, is_create_require_namespace_member,
  },
  visitors::{
    AtomMembers, ExportedVariableInfo, ExprRef, Identifier, VariableDeclaration, VariableInfo,
    VariableInfoFlags, get_non_optional_part,
  },
};

fn is_create_require_tag(tag: &str, include_create_require_fn: bool) -> bool {
  tag == CREATED_REQUIRE_IDENTIFIER_TAG
    || (include_create_require_fn
      && (tag == CREATE_REQUIRE_SPECIFIER_TAG || tag == CREATE_REQUIRE_EVALUATED_TAG))
}

impl JavascriptParser<'_> {
  fn in_block_scope<F>(&mut self, in_executed_path: bool, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;
    let old_in_try = self.in_try;
    let old_terminated = self.terminated;

    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(old_definitions);
    f(self);

    let terminated = self.terminated;

    self.definitions_db.exit_scope(self.definitions);
    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
    self.in_try = old_in_try;
    self.terminated = old_terminated;

    if in_executed_path && let Some(t) = terminated {
      self.terminated = Some(t);
    }
  }

  pub fn in_class_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = PatRef>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;
    let old_terminated = self.terminated;

    self.in_try = false;
    self.in_tagged_template_tag = false;
    self.terminated = None;
    self.definitions = self.definitions_db.create_child(old_definitions);

    if has_this {
      self.undefined_variable(&"this".into());
    }

    self.enter_patterns(params, |this, ident| {
      this.define_variable(Atom::from_ast(this.ast.ast, ident.name(this.ast.ast)));
    });

    f(self);

    self.in_try = old_in_try;
    self.definitions_db.exit_scope(self.definitions);
    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
    self.terminated = old_terminated;
  }

  pub(crate) fn in_function_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = PatRef>,
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;
    let old_terminated = self.terminated;

    self.definitions = self.definitions_db.create_child(old_definitions);
    self.in_tagged_template_tag = false;
    self.terminated = None;
    if has_this {
      self.undefined_variable(&"this".into());
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(Atom::from_ast(this.ast.ast, ident.name(this.ast.ast)));
    });
    f(self);

    self.definitions_db.exit_scope(self.definitions);
    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
    self.terminated = old_terminated;
  }

  pub fn walk_module_items(&mut self, statements: TypedSubRange<Stmt>) {
    let ast = self.ast.ast;
    for id in statements.iter() {
      let statement = ast.get_node_in_sub_range(id);
      self.walk_module_item(statement);
    }
  }

  fn walk_module_item(&mut self, statement: Stmt) {
    let ast = self.ast.ast;
    match ast.stmt_data(statement) {
      StmtData::ImportDeclaration(_)
      | StmtData::ExportNamedDeclaration(_)
      | StmtData::ExportDefaultDeclaration(_)
      | StmtData::ExportAllDeclaration(_)
      | StmtData::TsExportAssignment(_)
      | StmtData::TsNamespaceExportDeclaration(_) => {
        let drive = self.plugin_drive.clone();
        self.enter_statement(
          statement.span(ast),
          statement,
          |parser, node| drive.module_declaration(parser, node).unwrap_or_default(),
          |parser, node| match parser.ast.ast.stmt_data(node) {
            StmtData::ExportDefaultDeclaration(declaration) => {
              parser.walk_export_default_declaration(ExportDefaultDeclaration(declaration))
            }
            StmtData::ExportNamedDeclaration(declaration) => {
              if let Some(declaration) = declaration.declaration(parser.ast.ast) {
                parser.walk_statement(Statement::from_stmt(
                  parser.ast.ast,
                  Stmt::Declaration(declaration),
                ));
              }
            }
            _ => (),
          },
        );
      }
      _ => self.walk_statement(Statement::from_stmt(ast, statement)),
    }
  }

  fn walk_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
    let ast = self.ast.ast;
    match ast.export_default_declaration_kind_data(declaration.0.declaration(ast)) {
      ExportDefaultDeclarationKindData::Class(class) => {
        self.walk_statement(Statement::Class(MaybeNamedClassDecl(class)))
      }
      ExportDefaultDeclarationKindData::Function(function) => {
        self.walk_statement(Statement::Fn(MaybeNamedFunctionDecl(function)))
      }
      ExportDefaultDeclarationKindData::Expr(expression) => self.walk_expression(expression),
      _ => (),
    }
  }

  pub fn walk_statements(&mut self, statements: TypedSubRange<Stmt>) {
    let ast = self.ast.ast;
    let mut only_function_declaration = false;
    for id in statements.iter() {
      let statement = ast.get_node_in_sub_range(id);
      let stmt = Statement::from_stmt(ast, statement);
      if only_function_declaration
        && !matches!(stmt, Statement::Fn(_))
        && self
          .plugin_drive
          .clone()
          .unused_statement(self, stmt)
          .unwrap_or(false)
      {
        continue;
      }
      self.walk_statement(stmt);
      if self.terminated.is_some() {
        only_function_declaration = true;
      }
    }
  }

  pub(crate) fn walk_statement(&mut self, statement: Statement) {
    let drive = self.plugin_drive.clone();
    self.enter_statement(
      statement.span(self.ast.ast),
      statement,
      |parser, _| drive.statement(parser, statement).unwrap_or_default(),
      |parser, _| match statement {
        Statement::Block(stmt) => parser.walk_block_statement(stmt),
        Statement::Class(decl) => parser.walk_class_declaration(decl),
        Statement::Fn(decl) => parser.walk_function_declaration(decl),
        Statement::Var(decl) => parser.walk_variable_declaration(decl),
        Statement::DoWhile(stmt) => parser.walk_do_while_statement(stmt),
        Statement::Expr(stmt) => {
          // This is a bit different with webpack, so we can easily implement is_statement_level_expression
          // we didn't use pre_statement here like usual, this is referenced from walk_sequence_expression, which did the similar
          let old = parser.statement_path.pop().expect("should in statement");
          parser
            .statement_path
            .push(stmt.expression(parser.ast.ast).span(parser.ast.ast).into());
          parser.walk_expression_statement(stmt);
          parser.statement_path.pop();
          parser.statement_path.push(old);
        }
        Statement::ForIn(stmt) => parser.walk_for_in_statement(stmt),
        Statement::ForOf(stmt) => parser.walk_for_of_statement(stmt),
        Statement::For(stmt) => parser.walk_for_statement(stmt),
        Statement::If(stmt) => parser.walk_if_statement(stmt),
        Statement::Labeled(stmt) => parser.walk_labeled_statement(stmt),
        Statement::Return(stmt) => parser.walk_return_statement(stmt),
        Statement::Switch(stmt) => parser.walk_switch_statement(stmt),
        Statement::Throw(stmt) => parser.walk_throw_stmt(stmt),
        Statement::Try(stmt) => parser.walk_try_statement(stmt),
        Statement::While(stmt) => parser.walk_while_statement(stmt),
        Statement::With(stmt) => parser.walk_with_statement(stmt),
        _ => (),
      },
    );
  }

  fn walk_with_statement(&mut self, stmt: WithStatement) {
    self.in_block_scope(true, |this| {
      this.walk_expression(stmt.object(this.ast.ast));
      this.walk_nested_statement(stmt.body(this.ast.ast));
    });
  }

  fn walk_while_statement(&mut self, stmt: WhileStatement) {
    self.in_block_scope(false, |this| {
      this.walk_expression(stmt.test(this.ast.ast));
      this.walk_nested_statement(stmt.body(this.ast.ast));
    });
  }

  fn walk_try_statement(&mut self, stmt: TryStatement) {
    let ast = self.ast.ast;
    let block = stmt.block(ast);
    let was_in_try = self.in_try;
    if self.in_try {
      self.walk_statement(Statement::Block(block));
    } else {
      self.in_try = true;
      self.walk_statement(Statement::Block(block));
      self.in_try = false;
    }

    let try_terminated = self.terminated;
    self.terminated = None;

    let mut handler_terminated = None;
    if let Some(handler) = stmt.handler(ast) {
      self.walk_catch_clause(handler);
      handler_terminated = self.terminated;
      self.terminated = None;
    }

    let mut finalizer_terminated = None;
    if let Some(finalizer) = stmt.finalizer(ast) {
      self.walk_statement(Statement::Block(finalizer));
      finalizer_terminated = self.terminated;
      self.terminated = None;
    }

    if let Some(t) = finalizer_terminated {
      self.terminated = Some(t);
    } else if let Some(t) = try_terminated
      && (stmt.handler(ast).is_none() || handler_terminated.is_some())
    {
      self.terminated = handler_terminated.or(Some(t));
    }

    self.in_try = was_in_try;
  }

  fn walk_catch_clause(&mut self, catch_clause: CatchClause) {
    self.in_block_scope(true, |this| {
      let ast = this.ast.ast;
      if let Some(param) = catch_clause.param(ast) {
        this.enter_pattern(PatRef::Borrowed(param), |this, ident| {
          this.define_variable(Atom::from_ast(this.ast.ast, ident.name(this.ast.ast)));
        });
        this.walk_pattern(param)
      }
      let prev = this.prev_statement;
      let body = catch_clause.body(ast);
      this.block_pre_walk_statements(body.body(ast));
      this.prev_statement = prev;
      this.walk_statement(Statement::Block(body));
    })
  }

  fn walk_switch_statement(&mut self, stmt: SwitchStatement) {
    let ast = self.ast.ast;
    self.walk_expression(stmt.discriminant(ast));
    self.walk_switch_cases(stmt.cases(ast));
  }

  fn walk_switch_cases(&mut self, cases: TypedSubRange<SwitchCase>) {
    self.in_block_scope(false, |this| {
      let ast = this.ast.ast;
      for id in cases.iter() {
        let case = ast.get_node_in_sub_range(id);
        let consequent = case.consequent(ast);
        if !consequent.is_empty() {
          let prev = this.prev_statement;
          this.block_pre_walk_statements(consequent);
          this.prev_statement = prev;
        }
      }
      for id in cases.iter() {
        let case = ast.get_node_in_sub_range(id);
        if let Some(test) = case.test(ast) {
          this.walk_expression(test);
        }
        this.walk_statements(case.consequent(ast));
        this.terminated = None;
      }
    })
  }

  fn walk_return_statement(&mut self, stmt: ReturnStatement) {
    if let Some(arg) = stmt.argument(self.ast.ast) {
      self.walk_expression(arg);
    }
    if self.is_top_level_scope() {
      return;
    }
    // Mark current scope as terminated by return. This mirrors webpack's
    // `scope.terminated` behavior driven by `hooks.terminate`, which is
    // always tapped to `true` by its ConstPlugin for return statements.
    self.terminated = Some(ScopeTerminated::Return);
  }

  fn walk_throw_stmt(&mut self, stmt: ThrowStatement) {
    self.walk_expression(stmt.argument(self.ast.ast));
    if self.is_top_level_scope() {
      return;
    }
    // Same as above but for throw statements.
    self.terminated = Some(ScopeTerminated::Throw);
  }

  fn walk_labeled_statement(&mut self, stmt: LabeledStatement) {
    // TODO: self.hooks.label.get
    self.in_block_scope(false, |this| {
      this.walk_nested_statement(stmt.body(this.ast.ast));
    });
  }

  fn walk_if_statement(&mut self, stmt: IfStatement) {
    if let Some(result) = self.plugin_drive.clone().statement_if(self, stmt) {
      if result {
        self.walk_nested_statement(stmt.consequent(self.ast.ast));
      } else if let Some(alt) = stmt.alternate(self.ast.ast) {
        self.walk_nested_statement(alt);
      }
    } else {
      // Unknown or non-constant condition – walk the test for side effects and
      // both branches, only keeping termination when *both* are terminated.
      let guard = self.collect_dependencies_in_branch_guard(|parser| {
        parser.walk_expression(stmt.test(parser.ast.ast));
        let deps_in_guard = parser.dependencies_in_branch_guard.as_ref()?;
        if deps_in_guard.is_empty() {
          return None;
        }
        let evaluated = parser.evaluate_expression(stmt.test(parser.ast.ast));
        if evaluated.is_dependency() {
          return Some(evaluated.into_dependency());
        }
        None
      });

      if let Some(guard) = &guard {
        self.with_branch_guard(DependencyBranchGuard::new(guard.clone()), |this| {
          this.walk_nested_statement(stmt.consequent(this.ast.ast))
        });
      } else {
        self.walk_nested_statement(stmt.consequent(self.ast.ast));
      }
      let consequent_terminated = self.terminated;
      self.terminated = None;
      if let Some(alt) = stmt.alternate(self.ast.ast) {
        if let Some(guard) = guard {
          self.with_branch_guard(DependencyBranchGuard::new(guard.not()), |this| {
            this.walk_nested_statement(alt)
          });
        } else {
          self.walk_nested_statement(alt);
        }
      }
      let alternate_terminated = self.terminated;
      self.terminated = if consequent_terminated.is_some() && alternate_terminated.is_some() {
        alternate_terminated
      } else {
        None
      };
    }
  }

  fn walk_for_statement(&mut self, stmt: ForStatement) {
    self.in_block_scope(false, |this| {
      let ast = this.ast.ast;
      if let Some(init) = stmt.init(ast) {
        match ast.for_statement_init_data(init) {
          ForStatementInitData::VariableDeclaration(decl) => {
            let decl = VariableDeclaration(decl);
            this.block_pre_walk_variable_declaration(decl);
            this.prev_statement = None;
            this.walk_variable_declaration(decl);
          }
          ForStatementInitData::Expr(expr) => this.walk_expression(expr),
        }
      }
      if let Some(test) = stmt.test(ast) {
        this.walk_expression(test)
      }
      if let Some(update) = stmt.update(ast) {
        this.walk_expression(update)
      }
      let body = stmt.body(ast);
      if let Some(body) = body.as_block_statement(ast) {
        let statements = body.body(ast);
        let prev = this.prev_statement;
        this.block_pre_walk_statements(statements);
        this.prev_statement = prev;
        this.walk_statements(statements);
      } else {
        this.walk_nested_statement(body);
      }
    });
  }

  fn walk_for_of_statement(&mut self, stmt: ForOfStatement) {
    self.in_block_scope(false, |this| {
      let ast = this.ast.ast;
      let left = stmt.left(ast);
      this.walk_for_head(left);
      this.walk_expression(stmt.right(ast));
      if this.javascript_options.is_create_require_enabled() {
        this.clear_created_require_tags_in_for_head(left);
      }
      let body = stmt.body(ast);
      if let Some(body) = body.as_block_statement(ast) {
        let statements = body.body(ast);
        let prev = this.prev_statement;
        this.block_pre_walk_statements(statements);
        this.prev_statement = prev;
        this.walk_statements(statements);
      } else {
        this.walk_nested_statement(body);
      }
    });
  }

  fn walk_for_in_statement(&mut self, stmt: ForInStatement) {
    self.in_block_scope(false, |this| {
      let ast = this.ast.ast;
      let left = stmt.left(ast);
      this.walk_for_head(left);
      this.walk_expression(stmt.right(ast));
      if this.javascript_options.is_create_require_enabled() {
        this.clear_created_require_tags_in_for_head(left);
      }
      let body = stmt.body(ast);
      if let Some(body) = body.as_block_statement(ast) {
        let statements = body.body(ast);
        let prev = this.prev_statement;
        this.block_pre_walk_statements(statements);
        this.prev_statement = prev;
        this.walk_statements(statements);
      } else {
        this.walk_nested_statement(body);
      }
    });
  }

  fn walk_for_head(&mut self, for_head: ForStatementLeft) {
    match self.ast.ast.for_statement_left_data(for_head) {
      ForStatementLeftData::VariableDeclaration(decl) => {
        let decl = VariableDeclaration(decl);
        self.block_pre_walk_variable_declaration(decl);
        self.walk_variable_declaration(decl);
      }
      ForStatementLeftData::AssignmentTarget(target) => {
        self.walk_assignment_target(target);
      }
    }
  }

  fn clear_created_require_tags_in_for_head(&mut self, for_head: ForStatementLeft) {
    let ast = self.ast.ast;
    let ForStatementLeftData::AssignmentTarget(target) = ast.for_statement_left_data(for_head)
    else {
      return;
    };
    match ast.assignment_target_data(target) {
      AssignmentTargetData::SimpleAssignmentTarget(target) => {
        if let SimpleAssignmentTargetData::IdentifierReference(identifier) =
          ast.simple_assignment_target_data(target)
        {
          self.clear_create_require_tag(ast.get_utf8(identifier.name(ast)));
        }
      }
      AssignmentTargetData::ArrayAssignmentTarget(array) => {
        self.clear_created_require_tags_in_pattern(BindingPattern::ArrayPattern(array));
      }
      AssignmentTargetData::ObjectAssignmentTarget(object) => {
        self.clear_created_require_tags_in_pattern(BindingPattern::ObjectPattern(object));
      }
    }
  }

  fn walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    let drive = self.plugin_drive.clone();
    let ast = self.ast.ast;
    for declarator in decl.declarators(ast) {
      let init = declarator.init(ast);
      let id = declarator.id(ast);
      if self.javascript_options.is_create_require_enabled()
        && let Some(init) = init
        && let Some(assign) = init.as_assignment_expression(ast)
        && assign.operator(ast) == AssignmentOperator::Assign
        && let AssignmentTargetData::SimpleAssignmentTarget(simple) =
          ast.assignment_target_data(assign.left(ast))
        && let SimpleAssignmentTargetData::IdentifierReference(target) =
          ast.simple_assignment_target_data(simple)
        && let BindingPatternData::BindingIdentifier(binding) = ast.binding_pattern_data(id)
        && self
          .try_walk_created_require_assignment(assign, target)
          .unwrap_or_default()
      {
        self.copy_create_require_assignment_result(
          Atom::from(ast.get_utf8(binding.name(ast))),
          ast.get_utf8(target.name(ast)),
        );
        continue;
      }
      if let Some(init) = init
        && let Some(renamed_identifier) = self.get_rename_identifier(init)
        && let BindingPatternData::BindingIdentifier(ident) = ast.binding_pattern_data(id)
      {
        if self.javascript_options.is_create_require_enabled()
          && renamed_identifier == CREATE_REQUIRE_EVALUATED_TAG
          && !matches!(
            ast.expr_data(init),
            ExprData::CallExpression(_) | ExprData::NewExpression(_)
          )
        {
          self.set_variable(
            Atom::from(ast.get_utf8(ident.name(ast))),
            ExportedVariableInfo::Name(renamed_identifier),
          );
          self.walk_expression(init);
          continue;
        }
        if !(self.javascript_options.is_create_require_enabled()
          && renamed_identifier == CREATE_REQUIRE_EVALUATED_TAG
          && matches!(
            ast.expr_data(init),
            ExprData::CallExpression(_) | ExprData::NewExpression(_)
          ))
          && drive
            .can_rename(self, &renamed_identifier)
            .unwrap_or_default()
        {
          if !drive
            .rename(self, init, &renamed_identifier)
            .unwrap_or_default()
          {
            self.set_variable(
              Atom::from(ast.get_utf8(ident.name(ast))),
              ExportedVariableInfo::Name(renamed_identifier.clone()),
            );
          }
          continue;
        }
      }
      if !drive.declarator(self, declarator, decl).unwrap_or_default() {
        self.walk_pattern(id);
        if let Some(init) = init {
          self.walk_expression(init);
        }
      }
    }
  }

  fn walk_expression_statement(&mut self, stmt: ExpressionStatement) {
    self.walk_expression(stmt.expression(self.ast.ast));
  }

  pub fn walk_expression(&mut self, expr: Expr) {
    match self.ast.ast.expr_data(expr) {
      ExprData::ArrayExpression(expr) => self.walk_array_expression(expr),
      ExprData::ArrowFunctionExpression(expr) => self.walk_arrow_function_expression(expr),
      ExprData::AssignmentExpression(expr) => self.walk_assignment_expression(expr),
      ExprData::AwaitExpression(expr) => self.walk_await_expression(expr),
      ExprData::BinaryExpression(expr) => self.walk_binary_expression(expr),
      ExprData::LogicalExpression(expr) => self.walk_logical_expression(expr),
      ExprData::CallExpression(expr) => self.walk_call_expression(expr),
      ExprData::Class(expr) => self.walk_class_expression(expr),
      ExprData::ConditionalExpression(expr) => self.walk_conditional_expression(expr),
      ExprData::Function(expr) => self.walk_function_expression(expr),
      ExprData::IdentifierReference(expr) => self.walk_identifier(expr),
      ExprData::ImportExpression(expr) => self.walk_import_expression(expr),
      ExprData::MetaProperty(expr) => self.walk_meta_property(expr),
      ExprData::MemberExpression(expr) => self.walk_member_expression(expr),
      ExprData::NewExpression(expr) => self.walk_new_expression(expr),
      ExprData::ObjectExpression(expr) => self.walk_object_expression(expr),
      ExprData::ChainExpression(expr) => self.walk_chain_expression(expr),
      ExprData::SequenceExpression(expr) => self.walk_sequence_expression(expr),
      ExprData::TaggedTemplateExpression(expr) => self.walk_tagged_template_expression(expr),
      ExprData::TemplateLiteral(expr) => self.walk_template_expression(expr),
      ExprData::ThisExpression(expr) => self.walk_this_expression(expr),
      ExprData::UnaryExpression(expr) => self.walk_unary_expression(expr),
      ExprData::UpdateExpression(expr) => self.walk_update_expression(expr),
      ExprData::YieldExpression(expr) => self.walk_yield_expression(expr),
      ExprData::JsxElement(element) => {
        self.ensure_jsx_enabled();
        self.walk_jsx_element(element);
      }
      ExprData::JsxFragment(fragment) => {
        self.ensure_jsx_enabled();
        self.walk_jsx_fragment(fragment);
      }
      ExprData::ParenthesizedExpression(_) => unreachable!(),
      _ => (),
    }
  }

  fn walk_yield_expression(&mut self, expr: YieldExpression) {
    if let Some(argument) = expr.argument(self.ast.ast) {
      self.walk_expression(argument);
    }
  }

  fn walk_update_expression(&mut self, expr: UpdateExpression) {
    let ast = self.ast.ast;
    let argument = expr.argument(ast);
    if !self.javascript_options.is_create_require_enabled() {
      self.walk_simple_assign_target(argument);
      return;
    }
    let updated_ident = argument
      .as_identifier_reference(ast)
      .map(|ident| Atom::from(ast.get_utf8(ident.name(ast))));
    if let Some(name) = &updated_ident {
      self.clear_create_require_tag(name);
    }
    self.walk_simple_assign_target(argument);
    if let Some(name) = &updated_ident {
      self.clear_create_require_tag(name);
    }
  }

  fn clear_create_require_tag(&mut self, name: &str) {
    if let Some(variable_info) = self.get_variable_info(name) {
      let declared_scope = variable_info.declared_scope;
      let should_clear_name = variable_info.name.as_ref().is_some_and(|name| {
        name == CREATED_REQUIRE_IDENTIFIER_TAG
          || name == CREATE_REQUIRE_SPECIFIER_TAG
          || name == CREATE_REQUIRE_EVALUATED_TAG
      });
      let mut should_clear = should_clear_name;
      let mut tag_info_id = variable_info.tag_info;
      while let Some(id) = tag_info_id {
        let tag_info = self.definitions_db.expect_get_tag_info(id);
        if tag_info.tag == CREATED_REQUIRE_IDENTIFIER_TAG
          || tag_info.tag == CREATE_REQUIRE_SPECIFIER_TAG
          || tag_info.tag == CREATE_REQUIRE_EVALUATED_TAG
        {
          should_clear = true;
          break;
        }
        tag_info_id = tag_info.next;
      }
      if should_clear {
        let info = VariableInfo::create(
          &mut self.definitions_db,
          declared_scope,
          None,
          VariableInfoFlags::NORMAL,
          None,
        );
        self
          .definitions_db
          .set(declared_scope, Atom::from(name), info);
      }
    }
  }

  fn has_create_require_tag(&mut self, name: &str, include_create_require_fn: bool) -> bool {
    let Some(variable_info) = self.get_variable_info(name) else {
      return false;
    };
    if variable_info
      .name
      .as_ref()
      .is_some_and(|name| is_create_require_tag(name, include_create_require_fn))
    {
      return true;
    }
    let mut tag_info_id = variable_info.tag_info;
    while let Some(id) = tag_info_id {
      let tag_info = self.definitions_db.expect_get_tag_info(id);
      if is_create_require_tag(tag_info.tag, include_create_require_fn) {
        return true;
      }
      tag_info_id = tag_info.next;
    }
    false
  }

  fn clear_created_require_tags_in_pattern(&mut self, pattern: BindingPattern) {
    let ast = self.ast.ast;
    match ast.binding_pattern_data(pattern) {
      BindingPatternData::BindingIdentifier(identifier) => {
        self.clear_create_require_tag(ast.get_utf8(identifier.name(ast)))
      }
      BindingPatternData::SimpleAssignmentTarget(target) => {
        if let SimpleAssignmentTargetData::IdentifierReference(identifier) =
          ast.simple_assignment_target_data(target)
        {
          self.clear_create_require_tag(ast.get_utf8(identifier.name(ast)));
        }
      }
      BindingPatternData::AssignmentPattern(pattern) => {
        self.clear_created_require_tags_in_pattern(pattern.left(ast));
      }
      BindingPatternData::BindingRestElement(rest) => {
        self.clear_created_require_tags_in_pattern(rest.argument(ast));
      }
      BindingPatternData::ArrayPattern(array) => {
        for element in array
          .elements(ast)
          .iter()
          .filter_map(|id| ast.get_node_in_sub_range(id))
        {
          self.clear_created_require_tags_in_pattern(element);
        }
        if let Some(rest) = array.rest(ast) {
          self.clear_created_require_tags_in_pattern(rest.argument(ast));
        }
      }
      BindingPatternData::ObjectPattern(object) => {
        for property in object
          .properties(ast)
          .iter()
          .map(|id| ast.get_node_in_sub_range(id))
        {
          self.clear_created_require_tags_in_pattern(property.value(ast));
        }
        if let Some(rest) = object.rest(ast) {
          self.clear_created_require_tags_in_pattern(rest.argument(ast));
        }
      }
    }
  }

  #[cold]
  #[inline(never)]
  fn try_walk_created_require_assignment(
    &mut self,
    expr: AssignmentExpression,
    ident: IdentifierReference,
  ) -> Option<bool> {
    let ast = self.ast.ast;
    let ident_name = Atom::from(ast.get_utf8(ident.name(ast)));
    if matches!(
      expr.operator(ast),
      AssignmentOperator::LogicalOrAssign | AssignmentOperator::NullishAssign
    ) && self.has_create_require_tag(&ident_name, true)
    {
      return Some(true);
    }
    if expr.operator(ast) != AssignmentOperator::Assign {
      return None;
    }
    let right = expr.right(ast);
    if let Some(variable) = right.as_identifier_reference(ast).and_then(|rhs| {
      let rhs_name = Atom::from(ast.get_utf8(rhs.name(ast)));
      self
        .has_create_require_tag(&rhs_name, false)
        .then(|| self.get_variable_info(&rhs_name).map(|info| info.id()))
        .flatten()
    }) {
      self.set_variable(
        ident_name.clone(),
        ExportedVariableInfo::VariableInfo(variable),
      );
      return Some(true);
    }
    if let Some(rename_identifier) = self.get_rename_identifier(right)
      && let Some(data) = self
        .get_tag_data::<CreatedRequireTagData>(&rename_identifier, CREATED_REQUIRE_IDENTIFIER_TAG)
        .cloned()
    {
      self.tag_variable(
        ident_name.clone(),
        CREATED_REQUIRE_IDENTIFIER_TAG,
        Some(CreatedRequireTagData {
          side_effects: String::new(),
          ..data
        }),
      );
      if !right.is_identifier_reference(ast) {
        self.walk_expression(right);
      }
      return Some(true);
    }
    if is_create_require_namespace_member(self, right) {
      self.tag_variable_without_data(ident_name.clone(), CREATE_REQUIRE_SPECIFIER_TAG);
      self.walk_expression(right);
      return Some(true);
    }
    if let Some(rename_identifier) = self.get_rename_identifier(right)
      && rename_identifier == CREATE_REQUIRE_EVALUATED_TAG
    {
      self.set_variable(ident_name, ExportedVariableInfo::Name(rename_identifier));
      self.walk_expression(right);
      return Some(true);
    }
    None
  }

  fn copy_create_require_assignment_result(&mut self, binding: Atom, target: &str) {
    if let Some(data) = self
      .get_tag_data::<CreatedRequireTagData>(target, CREATED_REQUIRE_IDENTIFIER_TAG)
      .cloned()
    {
      self.tag_variable(
        binding,
        CREATED_REQUIRE_IDENTIFIER_TAG,
        Some(CreatedRequireTagData {
          side_effects: String::new(),
          ..data
        }),
      );
    } else if let Some(info) = self.get_variable_info(target)
      && info
        .name
        .as_ref()
        .is_some_and(|name| name == CREATE_REQUIRE_EVALUATED_TAG)
    {
      self.set_variable(
        binding,
        ExportedVariableInfo::Name(CREATE_REQUIRE_EVALUATED_TAG.into()),
      );
    } else if self.has_create_require_tag(target, true) {
      self.tag_variable_without_data(binding, CREATE_REQUIRE_SPECIFIER_TAG);
    }
  }

  fn walk_unary_expression(&mut self, expr: UnaryExpression) {
    let ast = self.ast.ast;
    let argument = expr.argument(ast);
    let drive = self.plugin_drive.clone();
    if expr.operator(ast) == UnaryOperator::Typeof
      && let Some(expr_info) =
        self.get_member_expression_info_from_expr(argument, AllowedMemberTypes::Expression)
    {
      let MemberExpressionInfo::Expression(expr_info) = expr_info else {
        // we use `AllowedMemberTypes::Expression` above
        unreachable!();
      };
      if expr_info
        .name
        .call_hooks_name(self, |this, for_name| drive.r#typeof(this, expr, for_name))
        .unwrap_or_default()
      {
        return;
      }
    };
    // TODO: expr.arg belongs chain_expression
    self.walk_expression(argument)
  }

  fn walk_this_expression(&mut self, expr: ThisExpression) {
    let drive = self.plugin_drive.clone();
    "this".call_hooks_name(self, |this, for_name| drive.this(this, expr, for_name));
  }

  pub(crate) fn walk_template_expression(&mut self, expr: TemplateLiteral) {
    let ast = self.ast.ast;
    self.walk_expressions(
      expr
        .expressions(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id)),
    );
  }

  fn walk_tagged_template_expression(&mut self, expr: TaggedTemplateExpression) {
    let ast = self.ast.ast;
    self.in_tagged_template_tag = true;
    self.walk_expression(expr.tag(ast));
    self.in_tagged_template_tag = false;

    self.walk_template_expression(expr.quasi(ast));
  }

  fn walk_sequence_expression(&mut self, expr: SequenceExpression) {
    let ast = self.ast.ast;
    let expressions = expr.expressions(ast);
    if self.is_statement_level_expression(expr.span(ast))
      && let Some(old) = self.statement_path.pop()
    {
      let prev = self.prev_statement;
      for expression in expressions.iter().map(|id| ast.get_node_in_sub_range(id)) {
        self.statement_path.push(expression.span(ast).into());
        self.walk_expression(expression);
        self.prev_statement = self.statement_path.pop();
      }
      self.prev_statement = prev;
      self.statement_path.push(old);
    } else {
      self.walk_expressions(expressions.iter().map(|id| ast.get_node_in_sub_range(id)));
    }
  }

  fn ensure_jsx_enabled(&self) {
    if !self.javascript_options.jsx.unwrap_or_default() {
      unreachable!();
    }
  }

  fn walk_jsx_element(&mut self, element: JsxElement) {
    let ast = self.ast.ast;
    let opening = element.opening_element(ast);
    self.walk_jsx_element_name(opening.name(ast));
    for attribute in opening
      .attributes(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      self.walk_jsx_attr_or_spread(attribute);
    }
    for child in element
      .children(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      self.walk_jsx_child(child);
    }
    if let Some(closing) = element.closing_element(ast) {
      self.walk_jsx_element_name(closing.name(ast));
    }
  }

  fn walk_jsx_fragment(&mut self, fragment: JsxFragment) {
    let ast = self.ast.ast;
    for child in fragment
      .children(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      self.walk_jsx_child(child);
    }
  }

  fn walk_jsx_child(&mut self, child: JsxChild) {
    let ast = self.ast.ast;
    match ast.jsx_child_data(child) {
      JsxChildData::JsxElement(element) => self.walk_jsx_element(element),
      JsxChildData::JsxFragment(fragment) => self.walk_jsx_fragment(fragment),
      JsxChildData::JsxExpressionContainer(container) => self.walk_jsx_expr_container(container),
      JsxChildData::JsxSpreadChild(spread) => self.walk_expression(spread.expression(ast)),
      JsxChildData::JsxText(_) => (),
    }
  }

  fn walk_jsx_expr_container(&mut self, container: JsxExpressionContainer) {
    let ast = self.ast.ast;
    match ast.jsx_expression_data(container.expression(ast)) {
      JsxExpressionData::Expr(expression) => self.walk_expression(expression),
      JsxExpressionData::JsxEmptyExpression(_) => (),
    }
  }

  fn walk_jsx_attr_or_spread(&mut self, attribute: JsxAttributeItem) {
    let ast = self.ast.ast;
    match ast.jsx_attribute_item_data(attribute) {
      JsxAttributeItemData::JsxAttribute(attribute) => self.walk_jsx_attr(attribute),
      JsxAttributeItemData::JsxSpreadAttribute(spread) => {
        self.walk_expression(spread.argument(ast));
      }
    }
  }

  fn walk_jsx_attr(&mut self, attribute: JsxAttribute) {
    if let Some(value) = attribute.value(self.ast.ast) {
      self.walk_jsx_attr_value(value);
    }
  }

  fn walk_jsx_attr_value(&mut self, value: JsxAttributeValue) {
    match self.ast.ast.jsx_attribute_value_data(value) {
      JsxAttributeValueData::StringLiteral(_) => (),
      JsxAttributeValueData::JsxExpressionContainer(container) => {
        self.walk_jsx_expr_container(container);
      }
      JsxAttributeValueData::JsxElement(element) => self.walk_jsx_element(element),
      JsxAttributeValueData::JsxFragment(fragment) => self.walk_jsx_fragment(fragment),
    }
  }

  fn walk_jsx_element_name(&mut self, name: JsxElementName) {
    match self.ast.ast.jsx_element_name_data(name) {
      JsxElementNameData::JsxIdentifier(identifier) => self.walk_jsx_identifier(identifier),
      JsxElementNameData::JsxMemberExpression(member) => self.walk_jsx_member_expr(member),
      JsxElementNameData::JsxNamespacedName(namespaced) => {
        self.walk_jsx_namespaced_name(namespaced);
      }
    }
  }

  fn walk_jsx_member_expr(&mut self, member: JsxMemberExpression) {
    let ast = self.ast.ast;
    let mut current = member;
    let mut members = AtomMembers::new();
    let mut members_optionals = OptionalMembers::new();
    let mut member_ranges = MemberRanges::new();
    let mut member_nodes = SmallVec::<[JsxMemberExpression; 2]>::new();
    let root = loop {
      let object = current.object(ast);
      members.push(Atom::from(ast.get_utf8(current.property(ast).name(ast))));
      members_optionals.push(false);
      member_ranges.push(object.span(ast));
      member_nodes.push(current);
      match ast.jsx_member_expression_object_data(object) {
        JsxMemberExpressionObjectData::JsxIdentifier(identifier) => break identifier,
        JsxMemberExpressionObjectData::JsxMemberExpression(member) => current = member,
      }
    };

    let root_name = Atom::from(ast.get_utf8(root.name(ast)));
    let Some(name_info) = self.get_name_info_from_variable(&root_name) else {
      self.walk_identifier_name(root_name, root.span(ast));
      return;
    };
    let resolved_root = name_info.name;
    let root_info = name_info.info.map_or_else(
      || ExportedVariableInfo::Name(root_name.clone()),
      |info| ExportedVariableInfo::VariableInfo(info.id()),
    );
    let name = object_and_members_to_name(resolved_root, &members);
    members.reverse();
    members_optionals.reverse();
    member_ranges.reverse();
    member_nodes.reverse();

    let expression_info = ExpressionExpressionInfo {
      name,
      root_info,
      members,
      members_optionals,
      member_ranges,
    };
    let drive = self.plugin_drive.clone();
    if expression_info
      .name
      .call_hooks_name(self, |this, for_name| {
        drive.member(this, member.into(), for_name)
      })
      .unwrap_or_default()
    {
      return;
    }
    if expression_info
      .root_info
      .call_hooks_name(self, |this, for_name| {
        drive.member_chain(
          this,
          member.into(),
          for_name,
          &expression_info.members,
          &expression_info.members_optionals,
          &expression_info.member_ranges,
        )
      })
      .unwrap_or_default()
    {
      return;
    }

    let mut prefix_name = expression_info.name.as_str();
    for index in (0..member_nodes.len().saturating_sub(1)).rev() {
      let removed_member = &expression_info.members[index + 1];
      prefix_name = &prefix_name[..prefix_name.len() - removed_member.len() - 1];
      if prefix_name
        .call_hooks_name(self, |this, for_name| {
          drive.member(this, member_nodes[index].into(), for_name)
        })
        .unwrap_or_default()
      {
        return;
      }
    }

    if !drive
      .unhandled_expression_member_chain(self, &expression_info.root_info, member.into())
      .unwrap_or_default()
    {
      self.walk_identifier_name(root_name, root.span(ast));
    }
  }

  fn walk_jsx_namespaced_name(&mut self, name: JsxNamespacedName) {
    let ast = self.ast.ast;
    self.walk_jsx_identifier(name.namespace(ast));
    self.walk_jsx_identifier(name.name(ast));
  }

  fn walk_jsx_identifier(&mut self, identifier: JsxIdentifier) {
    let ast = self.ast.ast;
    let name = ast.get_utf8(identifier.name(ast));
    // Intrinsic JSX names are not variable references.
    if name.as_bytes().first().is_some_and(u8::is_ascii_lowercase) {
      return;
    }
    self.walk_identifier_name(Atom::from(name), identifier.span(ast));
  }

  fn walk_object_expression(&mut self, expr: ObjectExpression) {
    let ast = self.ast.ast;
    for property in expr
      .properties(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      match ast.object_property_kind_data(property) {
        ObjectPropertyKindData::SpreadElement(spread) => {
          self.walk_expression(spread.argument(ast));
        }
        ObjectPropertyKindData::ObjectProperty(property) => {
          self.walk_object_property(property);
        }
      }
    }
  }

  fn walk_object_property(&mut self, property: ObjectProperty) {
    let ast = self.ast.ast;
    if property.computed(ast) {
      self.walk_property_key(property.key(ast));
    }
    if property.shorthand(ast) {
      self.in_short_hand = true;
      self.walk_expression(property.value(ast));
      self.in_short_hand = false;
    } else {
      self.walk_expression(property.value(ast));
    }
  }

  fn walk_property_key(&mut self, key: PropertyKey) {
    if let PropertyKeyData::Expr(expression) = self.ast.ast.property_key_data(key) {
      self.walk_expression(expression);
    }
  }

  fn walk_new_expression(&mut self, expr: NewExpression) {
    let ast = self.ast.ast;
    let callee = expr.callee(ast);
    if let Some(MemberExpressionInfo::Expression(info)) =
      self.get_member_expression_info_from_expr(callee, AllowedMemberTypes::Expression)
    {
      let result = if info.members.is_empty() {
        info.root_info.call_hooks_name(self, |parser, for_name| {
          parser
            .plugin_drive
            .clone()
            .new_expression(parser, expr, for_name)
        })
      } else {
        info.name.call_hooks_name(self, |parser, for_name| {
          parser
            .plugin_drive
            .clone()
            .new_expression(parser, expr, for_name)
        })
      };
      if result.unwrap_or_default() {
        return;
      }
    }
    self.walk_expression(callee);
    self.walk_arguments(
      expr
        .arguments(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id)),
    );
  }

  fn walk_meta_property(&mut self, expr: MetaProperty) {
    let ast = self.ast.ast;
    let Some(root_name) = expr.get_root_name(ast) else {
      unreachable!()
    };
    self
      .plugin_drive
      .clone()
      .meta_property(self, root_name, expr.span(ast));
  }

  fn walk_conditional_expression(&mut self, expr: ConditionalExpression) {
    let ast = self.ast.ast;
    let test = expr.test(ast);
    let consequent = expr.consequent(ast);
    let alternate = expr.alternate(ast);
    let result = self
      .plugin_drive
      .clone()
      .expression_conditional_operation(self, expr);

    if let Some(result) = result {
      if result {
        self.walk_expression(consequent);
      } else {
        self.walk_expression(alternate);
      }
    } else {
      let guard = self.collect_dependencies_in_branch_guard(|parser| {
        parser.walk_expression(test);
        let deps_in_guard = parser.dependencies_in_branch_guard.as_ref()?;
        if deps_in_guard.is_empty() {
          return None;
        }
        let evaluated = parser.evaluate_expression(test);
        if evaluated.is_dependency() {
          return Some(evaluated.into_dependency());
        }
        None
      });

      if let Some(guard) = &guard {
        self.with_branch_guard(DependencyBranchGuard::new(guard.clone()), |this| {
          this.walk_expression(consequent)
        });
      } else {
        self.walk_expression(consequent);
      }
      if let Some(guard) = guard {
        self.with_branch_guard(DependencyBranchGuard::new(guard.not()), |this| {
          this.walk_expression(alternate)
        });
      } else {
        self.walk_expression(alternate);
      }
    }
  }

  fn walk_class_expression(&mut self, expr: Class) {
    self.walk_class(expr, ClassDeclOrExpr::Expr(expr));
  }

  fn walk_chain_expression(&mut self, expr: ChainExpression) {
    if self
      .plugin_drive
      .clone()
      .optional_chaining(self, expr)
      .is_none()
    {
      self.enter_optional_chain(
        expr,
        |parser, call| parser.walk_call_expression(call),
        |parser, member| parser.walk_member_expression(member),
      );
    }
  }

  fn walk_member_expression(&mut self, expr: MemberExpression) {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    if let Some(expr_info) =
      self.get_member_expression_info(ExprRef::Member(expr), AllowedMemberTypes::all())
    {
      match expr_info {
        MemberExpressionInfo::Expression(expr_info) => {
          if expr_info
            .name
            .call_hooks_name(self, |this, for_name| {
              drive.member(this, expr.into(), for_name)
            })
            .unwrap_or_default()
          {
            return;
          }
          if expr_info
            .root_info
            .call_hooks_name(self, |this, for_name| {
              drive.member_chain(
                this,
                expr.into(),
                for_name,
                &expr_info.members,
                &expr_info.members_optionals,
                &expr_info.member_ranges,
              )
            })
            .unwrap_or_default()
          {
            return;
          }
          self.walk_member_expression_with_expression_name(
            expr,
            &expr_info.name,
            &expr_info.members,
            Some(|this: &mut Self| {
              drive.unhandled_expression_member_chain(this, &expr_info.root_info, expr.into())
            }),
          );
          return;
        }
        MemberExpressionInfo::Call(expr_info) => {
          if expr_info
            .root_info
            .call_hooks_name(self, |this, for_name| {
              drive.member_chain_of_call_member_chain(
                this,
                expr,
                &expr_info.callee_members,
                expr_info.call,
                &expr_info.members,
                &expr_info.member_ranges,
                for_name,
              )
            })
            .unwrap_or_default()
          {
            return;
          }
          self.walk_call_expression(expr_info.call);
          return;
        }
      }
    }

    let object = expr.object(ast);
    if object.is_meta_property(ast)
      && let Some(root_name) = object.get_root_name(ast)
    {
      let root_info = ExportedVariableInfo::Name(Atom::from(root_name));
      if drive
        .unhandled_expression_member_chain(self, &root_info, expr.into())
        .unwrap_or_default()
      {
        if expr.computed(ast)
          && let PropertyKeyData::Expr(property) = ast.property_key_data(expr.property(ast))
        {
          self.walk_expression(property);
        }
        return;
      }
    }

    // (await import(...)).a.b
    if let Some((call, members, await_expr)) = self.extract_await_import_member(expr) {
      if self.is_top_level_scope() {
        self
          .plugin_drive
          .clone()
          .top_level_await_expr(self, await_expr);
      }
      if self
        .plugin_drive
        .clone()
        .import_call(self, call, None, Some((&members, false)))
        .unwrap_or_default()
      {
        return;
      }
    }

    self.member_expr_in_optional_chain = false;
    self.walk_expression(object);
    if expr.computed(ast)
      && let PropertyKeyData::Expr(property) = ast.property_key_data(expr.property(ast))
    {
      self.walk_expression(property)
    }
  }

  fn walk_member_expression_with_expression_name<F>(
    &mut self,
    expr: MemberExpression,
    name: &str,
    members: &[Atom],
    on_unhandled: Option<F>,
  ) where
    F: FnOnce(&mut Self) -> Option<bool>,
  {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    let object = expr.object(ast);
    let member = match ast.expr_data(object) {
      ExprData::MemberExpression(member) => Some(member),
      ExprData::ChainExpression(chain) => chain.expression(ast).as_member_expression(ast),
      _ => None,
    };
    if let Some(member) = member
      && let Some(property) = members.last()
    {
      let origin = name.len();
      let name = &name[0..origin - 1 - property.len()];
      if name
        .call_hooks_name(self, |this, for_name| {
          drive.member(this, member.into(), for_name)
        })
        .unwrap_or_default()
      {
        return;
      }
      self.walk_member_expression_with_expression_name(
        member,
        name,
        &members[..members.len() - 1],
        on_unhandled,
      );
    } else if on_unhandled.is_none() {
      self.walk_expression(object);
    } else if let Some(on_unhandled) = on_unhandled
      && !on_unhandled(self).unwrap_or_default()
    {
      self.walk_expression(object);
    }

    if expr.computed(ast)
      && let PropertyKeyData::Expr(property) = ast.property_key_data(expr.property(ast))
    {
      self.walk_expression(property)
    }
  }

  fn property_key_name(ast: &Ast<'_>, key: PropertyKey) -> Option<Atom> {
    match ast.property_key_data(key) {
      PropertyKeyData::IdentifierName(identifier) => {
        Some(Atom::from(ast.get_utf8(identifier.name(ast))))
      }
      PropertyKeyData::StringLiteral(literal) => Some(Atom::from(
        ast.get_wtf8(literal.value(ast)).to_string_lossy().as_ref(),
      )),
      _ => None,
    }
  }

  fn formal_parameter_patterns<'a>(
    ast: &'a Ast<'a>,
    params: FormalParameters,
  ) -> impl Iterator<Item = BindingPattern> + 'a {
    params
      .items(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .filter_map(|item| item.as_formal_parameter(ast))
      .filter_map(|parameter| parameter.pattern(ast).as_binding_pattern(ast))
      .chain(params.rest(ast).map(BindingPattern::BindingRestElement))
  }

  fn parameter_identifiers<'a>(
    ast: &'a Ast<'a>,
    params: FormalParameters,
  ) -> impl Iterator<Item = Option<BindingIdentifier>> + 'a {
    params
      .items(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .map(|item| {
        item
          .as_formal_parameter(ast)?
          .pattern(ast)
          .as_binding_pattern(ast)?
          .as_binding_identifier(ast)
      })
  }

  fn has_simple_parameter_identifiers(ast: &Ast<'_>, params: FormalParameters) -> bool {
    params.rest(ast).is_none()
      && Self::parameter_identifiers(ast, params).all(|identifier| identifier.is_some())
  }

  pub(crate) fn walk_function_body(&mut self, body: FunctionBody) {
    let ast = self.ast.ast;
    for directive in body
      .directives(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      if ast.get_utf8(directive.value(ast)) == "use strict" {
        self.set_strict(true);
        break;
      }
    }
    let statements = body.body(ast);
    let prev = self.prev_statement;
    self.pre_walk_statements(statements);
    self.prev_statement = prev;
    self.block_pre_walk_statements(statements);
    self.prev_statement = prev;
    self.walk_statements(statements);
  }

  fn walk_import_expression(&mut self, expr: ImportExpression) {
    if self
      .plugin_drive
      .clone()
      .import_call(self, expr, None, None)
      .unwrap_or_default()
    {
      return;
    }
    let ast = self.ast.ast;
    self.walk_expression(expr.source(ast));
    if let Some(options) = expr.options(ast) {
      self.walk_expression(options);
    }
  }

  /// Walk IIFE function
  ///
  /// # Panics
  /// Either `Params` of `expr` or `params` passed in should be `BindingIdent`.
  fn _walk_iife(
    &mut self,
    expr: Expr,
    args: impl Iterator<Item = Argument>,
    current_this: Option<Argument>,
  ) {
    fn get_var_name(parser: &mut JavascriptParser, expr: Expr) -> Option<ExportedVariableInfo> {
      if let Some(rename_identifier) = parser.get_rename_identifier(expr)
        && let drive = parser.plugin_drive.clone()
        && rename_identifier
          .call_hooks_name(parser, |this, for_name| drive.can_rename(this, for_name))
          .unwrap_or_default()
      {
        if !rename_identifier
          .call_hooks_name(parser, |this, for_name| drive.rename(this, expr, for_name))
          .unwrap_or_default()
        {
          let variable = parser
            .get_variable_info(&rename_identifier)
            .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
            .unwrap_or(ExportedVariableInfo::Name(rename_identifier));
          return Some(variable);
        }
        return None;
      }
      parser.walk_expression(expr);
      None
    }

    fn get_argument_var_name(
      parser: &mut JavascriptParser,
      argument: Argument,
    ) -> Option<ExportedVariableInfo> {
      let ast = parser.ast.ast;
      match ast.argument_data(argument) {
        ArgumentData::Expr(expression) => get_var_name(parser, expression),
        ArgumentData::SpreadElement(spread) => {
          parser.walk_expression(spread.argument(ast));
          None
        }
      }
    }

    let ast = self.ast.ast;
    let rename_this = current_this.and_then(|this| get_argument_var_name(self, this));
    let variable_info_for_args = args
      .map(|argument| get_argument_var_name(self, argument))
      .collect::<Vec<_>>();

    let mut params = Vec::new();
    let mut scope_params = Vec::new();
    let formal_params = match ast.expr_data(expr) {
      ExprData::Function(function) => function.params(ast),
      ExprData::ArrowFunctionExpression(arrow) => arrow.params(ast),
      _ => unreachable!("IIFE must be a function or arrow function"),
    };
    for (i, identifier) in Self::parameter_identifiers(ast, formal_params)
      .map(|identifier| identifier.expect("IIFE parameters must be binding identifiers"))
      .enumerate()
    {
      params.push(identifier);
      if variable_info_for_args
        .get(i)
        .and_then(|info| info.as_ref())
        .is_none()
      {
        scope_params.push(PatRef::Borrowed(BindingPattern::BindingIdentifier(
          identifier,
        )));
      }
    }

    // Add function name in scope for recursive calls
    if let ExprData::Function(function) = ast.expr_data(expr)
      && let Some(identifier) = function.id(ast)
    {
      scope_params.push(PatRef::Owned(BindingPattern::BindingIdentifier(identifier)));
    }

    let was_top_level_scope = self.top_level_scope;
    self.top_level_scope = if !matches!(was_top_level_scope, TopLevelScope::False)
      && expr.is_arrow_function_expression(ast)
    {
      TopLevelScope::ArrowFunction
    } else {
      TopLevelScope::False
    };

    self.in_function_scope(true, scope_params.into_iter(), |parser| {
      if let Some(this) = rename_this
        && !expr.is_arrow_function_expression(parser.ast.ast)
      {
        parser.set_variable("this".into(), this)
      }
      for (i, var_info) in variable_info_for_args.into_iter().enumerate() {
        if let Some(var_info) = var_info
          && let Some(param) = params.get(i)
        {
          parser.set_variable(
            Atom::from(parser.ast.ast.get_utf8(param.name(parser.ast.ast))),
            var_info,
          );
        }
      }

      match parser.ast.ast.expr_data(expr) {
        ExprData::Function(function) => parser.walk_function_body(function.body(parser.ast.ast)),
        ExprData::ArrowFunctionExpression(arrow) => {
          match parser
            .ast
            .ast
            .arrow_function_body_data(arrow.body(parser.ast.ast))
          {
            ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
            ArrowFunctionBodyData::Expr(expression) => parser.walk_expression(expression),
          }
        }
        _ => unreachable!(),
      }
    });
    self.top_level_scope = was_top_level_scope;
  }

  fn walk_call_expression(&mut self, expr: CallExpression) {
    let ast = self.ast.ast;
    let callee = expr.callee(ast);
    let arguments = expr.arguments(ast);

    if let Some(member) = callee.as_member_expression(ast)
      && let Some(function) = member.object(ast).as_function(ast)
      && Self::property_key_name(ast, member.property(ast))
        .is_some_and(|name| name == "call" || name == "bind")
      && !arguments.is_empty()
      && Self::has_simple_parameter_identifiers(ast, function.params(ast))
    {
      let mut args = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
      let current_this = args.next();
      self._walk_iife(member.object(ast), args, current_this);
      return;
    }

    let direct_params = match ast.expr_data(callee) {
      ExprData::Function(function) => Some(function.params(ast)),
      ExprData::ArrowFunctionExpression(arrow) => Some(arrow.params(ast)),
      _ => None,
    };
    if direct_params.is_some_and(|params| Self::has_simple_parameter_identifiers(ast, params)) {
      self._walk_iife(
        callee,
        arguments.iter().map(|id| ast.get_node_in_sub_range(id)),
        None,
      );
      return;
    }

    if let Some(member) = callee.as_member_expression(ast) {
      if let Some(MemberExpressionInfo::Call(expr_info)) =
        self.get_member_expression_info(ExprRef::Member(member), AllowedMemberTypes::CallExpression)
        && expr_info
          .root_info
          .call_hooks_name(self, |this, for_name| {
            this
              .plugin_drive
              .clone()
              .call_member_chain_of_call_member_chain(
                this,
                expr,
                &expr_info.callee_members,
                expr_info.call,
                &expr_info.members,
                &expr_info.member_ranges,
                for_name,
              )
          })
          .unwrap_or_default()
      {
        return;
      }
      // import(...).then(...)
      if let Some(import) = member.object(ast).as_import_expression(ast)
        && Self::property_key_name(ast, member.property(ast)).as_deref() == Some("then")
        && self
          .plugin_drive
          .clone()
          .import_call(self, import, Some(expr), None)
          .unwrap_or_default()
      {
        return;
      }
      // (await import(...)).a.b()
      if let Some((call, members, await_expr)) = self.extract_await_import_member(member) {
        if self.is_top_level_scope() {
          self
            .plugin_drive
            .clone()
            .top_level_await_expr(self, await_expr);
        }
        if self
          .plugin_drive
          .clone()
          .import_call(self, call, None, Some((&members, true)))
          .unwrap_or_default()
        {
          self.walk_arguments(arguments.iter().map(|id| ast.get_node_in_sub_range(id)));
          return;
        }
      }
    }
    let evaluated_callee = self.evaluate_expression(callee);
    if evaluated_callee.is_identifier() {
      let members = evaluated_callee.members().map_or(&[][..], Vec::as_slice);
      let owned_members_optionals;
      let members_optionals = match evaluated_callee.members_optionals() {
        Some(members_optionals) => members_optionals.as_slice(),
        None => {
          owned_members_optionals =
            std::iter::repeat_n(false, members.len()).collect::<OptionalMembers>();
          owned_members_optionals.as_slice()
        }
      };
      let member_ranges = evaluated_callee
        .member_ranges()
        .map_or(&[][..], Vec::as_slice);
      let drive = self.plugin_drive.clone();
      if evaluated_callee
        .root_info()
        .call_hooks_name(self, |parser, for_name| {
          drive.call_member_chain(
            parser,
            expr,
            for_name,
            members,
            members_optionals,
            member_ranges,
          )
        })
        .unwrap_or_default()
      {
        /* result1 */
        return;
      }

      if drive
        .call(self, expr, evaluated_callee.identifier())
        .unwrap_or_default()
      {
        /* result2 */
        return;
      }
    }

    if let Some(member) = callee.as_member_expression(ast) {
      self.walk_expression(member.object(ast));
      if member.computed(ast)
        && let PropertyKeyData::Expr(property) = ast.property_key_data(member.property(ast))
      {
        self.walk_expression(property);
      }
    } else {
      self.walk_expression(callee);
    }
    self.walk_arguments(arguments.iter().map(|id| ast.get_node_in_sub_range(id)));
  }

  fn extract_await_import_member(
    &self,
    expr: MemberExpression,
  ) -> Option<(ImportExpression, AtomMembers, AwaitExpression)> {
    let ast = self.ast.ast;
    let super::RawExtractedMemberExpressionChainData {
      object,
      members,
      mut members_optionals,
      ..
    } = self.extract_member_expression_chain_raw(ExprRef::Member(expr));
    let ExprRef::Await(await_expr) = object else {
      return None;
    };
    let call = await_expr.argument(ast).as_import_expression(ast)?;
    let mut members = super::materialize_member_atoms(ast, members);
    members.reverse();
    members_optionals.reverse();
    let members = get_non_optional_part(&members, &members_optionals);
    Some((call, members.into(), await_expr))
  }

  pub fn walk_arguments<I>(&mut self, arguments: I)
  where
    I: Iterator<Item = Argument>,
  {
    let ast = self.ast.ast;
    for argument in arguments {
      match ast.argument_data(argument) {
        ArgumentData::Expr(expression) => self.walk_expression(expression),
        ArgumentData::SpreadElement(spread) => self.walk_expression(spread.argument(ast)),
      }
    }
  }

  fn walk_left_right_expression(&mut self, left: Expr, right: Expr) {
    self.walk_expression(left);
    self.walk_expression(right);
  }

  fn walk_binary_expression(&mut self, expr: BinaryExpression) {
    if self
      .plugin_drive
      .clone()
      .binary_expression(self, expr)
      .is_none()
    {
      let ast = self.ast.ast;
      self.walk_left_right_expression(expr.left(ast), expr.right(ast));
    }
  }

  fn walk_logical_expression(&mut self, expr: LogicalExpression) {
    let ast = self.ast.ast;
    if let Some(keep_right) = self
      .plugin_drive
      .clone()
      .expression_logical_operator(self, expr)
    {
      if keep_right {
        self.walk_expression(expr.right(ast));
      }
    } else {
      self.walk_left_right_expression(expr.left(ast), expr.right(ast));
    }
  }

  fn walk_await_expression(&mut self, expr: AwaitExpression) {
    if self.is_top_level_scope() {
      self.plugin_drive.clone().top_level_await_expr(self, expr);
    }
    self.walk_expression(expr.argument(self.ast.ast));
  }

  fn walk_identifier(&mut self, identifier: IdentifierReference) {
    let ast = self.ast.ast;
    let name = ast.get_utf8(identifier.name(ast));
    let span = identifier.span(ast);
    let drive = self.plugin_drive.clone();
    name.call_hooks_name(self, |this, for_name| {
      drive.identifier(this, &Identifier { span }, for_name)
    });
  }

  fn walk_identifier_name(&mut self, name: Atom, span: Span) {
    let drive = self.plugin_drive.clone();
    name.call_hooks_name(self, |this, for_name| {
      drive.identifier(this, &Identifier { span }, for_name)
    });
  }

  fn get_rename_identifier(&mut self, expr: Expr) -> Option<Atom> {
    let result = self.evaluate_expression(expr);
    result.is_identifier().then(|| result.identifier().clone())
  }

  fn walk_assignment_expression(&mut self, expr: AssignmentExpression) {
    let ast = self.ast.ast;
    let left = expr.left(ast);
    let right = expr.right(ast);
    let drive = self.plugin_drive.clone();
    if let Some(simple) = left.as_simple_assignment_target(ast)
      && let Some(ident) = simple.as_identifier_reference(ast)
    {
      if self.javascript_options.is_create_require_enabled()
        && self
          .try_walk_created_require_assignment(expr, ident)
          .unwrap_or_default()
      {
        return;
      }
      if expr.operator(ast) == AssignmentOperator::Assign
        && let Some(rename_identifier) = self.get_rename_identifier(right)
        && rename_identifier
          .call_hooks_name(self, |this, for_name| drive.can_rename(this, for_name))
          .unwrap_or_default()
      {
        if !rename_identifier
          .call_hooks_name(self, |this, for_name| drive.rename(this, right, for_name))
          .unwrap_or_default()
        {
          let variable = self
            .get_variable_info(&rename_identifier)
            .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
            .unwrap_or(ExportedVariableInfo::Name(rename_identifier));
          self.set_variable(Atom::from(ast.get_utf8(ident.name(ast))), variable);
        }
        return;
      }
      if !self.javascript_options.is_create_require_enabled()
        || !right
          .as_identifier_reference(ast)
          .is_some_and(|rhs| self.has_create_require_tag(ast.get_utf8(rhs.name(ast)), false))
      {
        self.walk_expression(right);
      }
      let name = ast.get_utf8(ident.name(ast));
      if self.javascript_options.is_create_require_enabled() {
        // The assignment target already gives us the canonical identifier
        // name. Clear any createRequire-derived tag here instead of trying to
        // reconstruct the name from a hook-facing span.
        self.clear_create_require_tag(name);
      }
      if !name
        .call_hooks_name(self, |this, for_name| {
          drive.assign(
            this,
            expr,
            &Identifier {
              span: ident.span(ast),
            },
            for_name,
          )
        })
        .unwrap_or_default()
      {
        self.walk_identifier(ident);
      }
    } else if let Some(array) = left.as_array_assignment_target(ast) {
      self.walk_expression(right);
      if self.javascript_options.is_create_require_enabled() {
        self.clear_created_require_tags_in_pattern(BindingPattern::ArrayPattern(array));
      }
      self.enter_array_pattern(array, |this, ident| {
        let ast = this.ast.ast;
        let name = ast.get_utf8(ident.name(ast));
        if !name
          .call_hooks_name(this, |this, for_name| {
            drive.assign(
              this,
              expr,
              &Identifier {
                span: ident.span(ast),
              },
              for_name,
            )
          })
          .unwrap_or_default()
        {
          this.define_variable(Atom::from(name));
        }
      });
      self.walk_array_pattern(array);
    } else if let Some(object) = left.as_object_assignment_target(ast) {
      self.walk_expression(right);
      if self.javascript_options.is_create_require_enabled() {
        self.clear_created_require_tags_in_pattern(BindingPattern::ObjectPattern(object));
      }
      self.enter_object_pattern(object, |this, ident| {
        let ast = this.ast.ast;
        let name = ast.get_utf8(ident.name(ast));
        if !name
          .call_hooks_name(this, |this, for_name| {
            drive.assign(
              this,
              expr,
              &Identifier {
                span: ident.span(ast),
              },
              for_name,
            )
          })
          .unwrap_or_default()
        {
          this.define_variable(Atom::from(name));
        }
      });
      self.walk_object_pattern(object);
    } else if let Some(member) = left
      .as_simple_assignment_target(ast)
      .and_then(|target| target.as_member_expression(ast))
    {
      if let Some(MemberExpressionInfo::Expression(expr_name)) =
        self.get_member_expression_info(ExprRef::Member(member), AllowedMemberTypes::Expression)
        && expr_name
          .root_info
          .call_hooks_name(self, |parser, for_name| {
            drive.assign_member_chain(
              parser,
              expr,
              &expr_name.members,
              &expr_name.member_ranges,
              for_name,
            )
          })
          .unwrap_or_default()
      {
        return;
      }
      self.walk_expression(right);
      self.walk_assignment_target(left);
    } else {
      self.walk_expression(right);
      self.walk_assignment_target(left);
    }
  }

  fn walk_arrow_function_expression(&mut self, expr: ArrowFunctionExpression) {
    let ast = self.ast.ast;
    let params = expr.params(ast);
    let was_top_level_scope = self.top_level_scope;
    if !matches!(was_top_level_scope, TopLevelScope::False) {
      self.top_level_scope = TopLevelScope::ArrowFunction;
    }
    self.in_function_scope(
      false,
      Self::formal_parameter_patterns(ast, params).map(PatRef::Borrowed),
      |this| {
        for pattern in Self::formal_parameter_patterns(ast, params) {
          this.walk_pattern(pattern)
        }
        match this
          .ast
          .ast
          .arrow_function_body_data(expr.body(this.ast.ast))
        {
          ArrowFunctionBodyData::FunctionBody(body) => this.walk_function_body(body),
          ArrowFunctionBodyData::Expr(expression) => this.walk_expression(expression),
        }
      },
    );
    self.top_level_scope = was_top_level_scope;
  }

  fn walk_expressions<I>(&mut self, expressions: I)
  where
    I: Iterator<Item = Expr>,
  {
    for expr in expressions {
      self.walk_expression(expr)
    }
  }

  fn walk_array_expression(&mut self, expr: ArrayExpression) {
    let ast = self.ast.ast;
    self.walk_arguments(
      expr
        .elements(ast)
        .iter()
        .filter_map(|id| ast.get_node_in_sub_range(id)),
    );
  }

  fn walk_nested_statement(&mut self, stmt: Stmt) {
    self.prev_statement = None;
    self.walk_statement(Statement::from_stmt(self.ast.ast, stmt));
  }

  fn walk_do_while_statement(&mut self, stmt: DoWhileStatement) {
    let ast = self.ast.ast;
    self.walk_nested_statement(stmt.body(ast));
    self.walk_expression(stmt.test(ast));
  }

  fn walk_block_statement(&mut self, stmt: BlockStatement) {
    let ast = self.ast.ast;
    let statements = stmt.body(ast);
    self.in_block_scope(true, |this| {
      let prev = this.prev_statement;
      this.block_pre_walk_statements(statements);
      this.prev_statement = prev;
      this.walk_statements(statements);
    })
  }

  fn walk_function_declaration(&mut self, decl: MaybeNamedFunctionDecl) {
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    let ast = self.ast.ast;
    let function = decl.function();
    let patterns = Self::formal_parameter_patterns(ast, function.params(ast));
    self.in_function_scope(true, patterns.map(PatRef::Borrowed), |this| {
      this.walk_function(function);
    });
    self.top_level_scope = was_top_level;
  }

  fn walk_function(&mut self, function: Function) {
    let ast = self.ast.ast;
    for pattern in Self::formal_parameter_patterns(ast, function.params(ast)) {
      self.walk_pattern(pattern)
    }
    self.walk_function_body(function.body(ast));
  }

  fn walk_function_expression(&mut self, expr: Function) {
    let ast = self.ast.ast;
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    let scope_params = Self::formal_parameter_patterns(ast, expr.params(ast))
      .map(PatRef::Borrowed)
      .chain(
        expr
          .id(ast)
          .map(BindingPattern::BindingIdentifier)
          .map(PatRef::Owned),
      );

    self.in_function_scope(true, scope_params, |this| {
      this.walk_function(expr);
    });
    self.top_level_scope = was_top_level;
  }

  pub fn walk_pattern(&mut self, pattern: BindingPattern) {
    match self.ast.ast.binding_pattern_data(pattern) {
      BindingPatternData::ArrayPattern(array) => self.walk_array_pattern(array),
      BindingPatternData::AssignmentPattern(assignment) => {
        self.walk_assignment_pattern(assignment);
      }
      BindingPatternData::ObjectPattern(object) => self.walk_object_pattern(object),
      BindingPatternData::BindingRestElement(rest) => self.walk_rest_element(rest),
      BindingPatternData::SimpleAssignmentTarget(target) => self.walk_simple_assign_target(target),
      BindingPatternData::BindingIdentifier(_) => (),
    }
  }

  fn walk_simple_assign_target(&mut self, target: SimpleAssignmentTarget) {
    let ast = self.ast.ast;
    match ast.simple_assignment_target_data(target) {
      SimpleAssignmentTargetData::IdentifierReference(identifier) => {
        self.walk_identifier(identifier);
      }
      SimpleAssignmentTargetData::MemberExpression(member) => self.walk_member_expression(member),
      SimpleAssignmentTargetData::TsAsExpression(expression) => {
        self.walk_expression(expression.expression(ast));
      }
      SimpleAssignmentTargetData::TsSatisfiesExpression(expression) => {
        self.walk_expression(expression.expression(ast));
      }
      SimpleAssignmentTargetData::TsTypeAssertion(expression) => {
        self.walk_expression(expression.expression(ast));
      }
      SimpleAssignmentTargetData::TsNonNullExpression(expression) => {
        self.walk_expression(expression.expression(ast));
      }
    }
  }

  fn walk_assignment_target(&mut self, target: AssignmentTarget) {
    match self.ast.ast.assignment_target_data(target) {
      AssignmentTargetData::SimpleAssignmentTarget(target) => {
        self.walk_simple_assign_target(target);
      }
      AssignmentTargetData::ArrayAssignmentTarget(array) => self.walk_array_pattern(array),
      AssignmentTargetData::ObjectAssignmentTarget(object) => self.walk_object_pattern(object),
    }
  }

  fn walk_rest_element(&mut self, rest: BindingRestElement) {
    self.walk_pattern(rest.argument(self.ast.ast));
  }

  fn walk_object_pattern(&mut self, object: ObjectPattern) {
    let ast = self.ast.ast;
    for property in object
      .properties(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      if property.computed(ast) {
        self.walk_property_key(property.key(ast));
      }
      self.walk_pattern(property.value(ast));
    }
    if let Some(rest) = object.rest(ast) {
      self.walk_rest_element(rest);
    }
  }

  fn walk_assignment_pattern(&mut self, pattern: AssignmentPattern) {
    let ast = self.ast.ast;
    self.walk_expression(pattern.right(ast));
    self.walk_pattern(pattern.left(ast));
  }

  fn walk_array_pattern(&mut self, pattern: ArrayPattern) {
    let ast = self.ast.ast;
    for element in pattern
      .elements(ast)
      .iter()
      .filter_map(|id| ast.get_node_in_sub_range(id))
    {
      self.walk_pattern(element);
    }
    if let Some(rest) = pattern.rest(ast) {
      self.walk_rest_element(rest);
    }
  }

  fn walk_class_declaration(&mut self, decl: MaybeNamedClassDecl) {
    self.walk_class(decl.class(), ClassDeclOrExpr::Decl(decl.class()));
  }

  fn walk_class(&mut self, classy: Class, class_decl_or_expr: ClassDeclOrExpr) {
    let ast = self.ast.ast;
    if let Some(super_class) = classy.super_class(ast)
      && !self
        .plugin_drive
        .clone()
        .class_extends_expression(self, super_class, class_decl_or_expr)
        .unwrap_or_default()
    {
      self.walk_expression(super_class);
    }

    // TODO: define variable for class expression in block pre walk
    let scope_param = match class_decl_or_expr {
      ClassDeclOrExpr::Expr(class_expr) => class_expr
        .id(ast)
        .map(BindingPattern::BindingIdentifier)
        .map(PatRef::Owned),
      ClassDeclOrExpr::Decl(_) => None,
    };

    let elements = classy.body(ast).body(ast);
    self.in_class_scope(true, scope_param.into_iter(), |this| {
      for class_element in elements.iter().map(|id| ast.get_node_in_sub_range(id)) {
        if this
          .plugin_drive
          .clone()
          .class_body_element(this, class_element, class_decl_or_expr)
          .unwrap_or_default()
        {
          continue;
        }

        match this.ast.ast.class_element_data(class_element) {
          ClassElementData::MethodDefinition(method) => {
            let ast = this.ast.ast;
            if method.computed(ast) {
              this.walk_property_key(method.key(ast));
            }
            if this
              .plugin_drive
              .clone()
              .class_body_value(this, class_element, method.span(ast), class_decl_or_expr)
              .unwrap_or_default()
            {
              continue;
            }
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            let function = method.value(ast);
            let patterns = Self::formal_parameter_patterns(ast, function.params(ast));
            this.in_function_scope(true, patterns.map(PatRef::Borrowed), |this| {
              this.walk_function(function)
            });
            this.top_level_scope = was_top_level;
          }
          ClassElementData::PropertyDefinition(property) => {
            let ast = this.ast.ast;
            if property.computed(ast) {
              this.walk_property_key(property.key(ast));
            }
            if let Some(value) = property.value(ast)
              && !this
                .plugin_drive
                .clone()
                .class_body_value(this, class_element, value.span(ast), class_decl_or_expr)
                .unwrap_or_default()
            {
              let was_top_level = this.top_level_scope;
              this.top_level_scope = TopLevelScope::False;
              this.walk_expression(value);
              this.top_level_scope = was_top_level;
            }
          }
          ClassElementData::StaticBlock(block) => {
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            let ast = this.ast.ast;
            let statements = block.body(ast);
            this.in_block_scope(true, |this| {
              let prev = this.prev_statement;
              this.block_pre_walk_statements(statements);
              this.prev_statement = prev;
              this.walk_statements(statements);
            });
            this.top_level_scope = was_top_level;
          }
          ClassElementData::TsMethodDefinition(method) => {
            if method.computed(this.ast.ast) {
              this.walk_property_key(method.key(this.ast.ast));
            }
          }
          ClassElementData::TsIndexSignature(_) => {}
        };
      }
    });
  }
}
