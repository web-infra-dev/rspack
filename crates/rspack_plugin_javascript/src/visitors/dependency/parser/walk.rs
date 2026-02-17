use std::borrow::Cow;

use itertools::Itertools;
use swc_core::atoms::Atom;
use swc_experimental_ecma_ast::{
  ArrayLit, ArrayPat, ArrowExpr, AssignExpr, AssignPat, AssignTarget, AssignTargetPat, Ast,
  AwaitExpr, BinExpr, BlockStmt, BlockStmtOrExpr, CallExpr, Callee, CatchClause, Class, ClassExpr,
  ClassMember, CondExpr, DefaultDecl, DoWhileStmt, ExportDefaultDecl, Expr, ExprOrSpread, ExprStmt,
  FnExpr, ForHead, ForInStmt, ForOfStmt, ForStmt, Function, GetSpan, GetterProp, Ident, IdentName,
  IfStmt, JSXAttr, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXElementChild, JSXElementName,
  JSXExpr, JSXExprContainer, JSXFragment, JSXMemberExpr, JSXNamespacedName, JSXObject,
  KeyValueProp, LabeledStmt, MemberExpr, MemberProp, MetaPropExpr, ModuleDecl, ModuleItem, NewExpr,
  ObjectLit, ObjectPat, ObjectPatProp, OptCall, OptChainExpr, Param, Pat, Prop, PropName,
  PropOrSpread, RestPat, ReturnStmt, SeqExpr, SetterProp, SimpleAssignTarget, Stmt, SwitchCase,
  SwitchStmt, TaggedTpl, ThisExpr, ThrowStmt, Tpl, TryStmt, TypedSubRange, UnaryExpr, UnaryOp,
  UpdateExpr, VarDeclOrExpr, WhileStmt, WithStmt, YieldExpr,
};

use super::{
  AllowedMemberTypes, CallHooksName, JavascriptParser, MemberExpressionInfo, RootName,
  TopLevelScope,
  estree::{ClassDeclOrExpr, MaybeNamedClassDecl, MaybeNamedFunctionDecl, Statement},
};
use crate::{
  parser_plugin::{JavascriptParserPlugin, is_logic_op},
  visitors::{ExportedVariableInfo, VariableDeclaration},
};

fn warp_ident_to_pat(ast: &mut Ast, ident: Ident) -> Pat {
  Pat::Ident(ast.binding_ident(ident.span(ast), ident))
}

impl JavascriptParser<'_> {
  fn in_block_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(old_definitions);
    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub fn in_class_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Pat>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_try = false;
    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(old_definitions);

    if has_this {
      self.undefined_variable(&"this".into());
    }

    self.enter_patterns(params, |this, ident| {
      this.define_variable(this.ast.get_atom(ident.sym(&this.ast)));
    });

    f(self);

    self.in_try = old_in_try;
    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub(crate) fn in_function_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Pat>,
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.definitions = self.definitions_db.create_child(old_definitions);
    self.in_tagged_template_tag = false;
    if has_this {
      self.undefined_variable(&"this".into());
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(this.ast.get_atom(ident.sym(&this.ast)));
    });
    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub fn walk_module_items(&mut self, statements: TypedSubRange<ModuleItem>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.walk_module_item(statement);
    }
  }

  fn walk_module_item(&mut self, statement: ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(m) => {
        self.enter_statement(
          m,
          |parser, m| {
            parser
              .plugin_drive
              .clone()
              .module_declaration(parser, m)
              .unwrap_or_default()
          },
          |parser, m| match m {
            ModuleDecl::ExportDefaultDecl(decl) => {
              parser.walk_export_default_declaration(decl);
            }
            ModuleDecl::ExportDecl(decl) => {
              parser.walk_statement(Statement::from_decl(decl.decl(&parser.ast), &parser.ast))
            }
            ModuleDecl::ExportDefaultExpr(expr) => parser.walk_expression(expr.expr(&parser.ast)),
            ModuleDecl::ExportAll(_) | ModuleDecl::ExportNamed(_) | ModuleDecl::Import(_) => (),
          },
        );
      }
      ModuleItem::Stmt(s) => self.walk_statement(Statement::from_stmt(s, &self.ast)),
    }
  }

  fn walk_export_default_declaration(&mut self, decl: ExportDefaultDecl) {
    match decl.decl(&self.ast) {
      DefaultDecl::Class(c) => self.walk_statement(Statement::Class(
        MaybeNamedClassDecl::from_class_expr(c, &self.ast),
      )),
      DefaultDecl::Fn(f) => self.walk_statement(Statement::Fn(
        MaybeNamedFunctionDecl::from_fn_expr(f, &self.ast),
      )),
    }
  }

  pub fn walk_statements(&mut self, statements: TypedSubRange<Stmt>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.walk_statement(Statement::from_stmt(statement, &self.ast));
    }
  }

  pub(crate) fn walk_statement(&mut self, statement: Statement) {
    self.enter_statement(
      statement,
      |parser, _| {
        parser
          .plugin_drive
          .clone()
          .statement(parser, statement)
          .unwrap_or_default()
      },
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
            .push(stmt.expr(&parser.ast).span(&parser.ast).into());
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

  fn walk_with_statement(&mut self, stmt: WithStmt) {
    self.walk_expression(stmt.obj(&self.ast));
    self.walk_nested_statement(stmt.body(&self.ast));
  }

  fn walk_while_statement(&mut self, stmt: WhileStmt) {
    self.walk_expression(stmt.test(&self.ast));
    self.walk_nested_statement(stmt.body(&self.ast));
  }

  fn walk_try_statement(&mut self, stmt: TryStmt) {
    if self.in_try {
      self.walk_statement(Statement::Block(stmt.block(&self.ast)));
    } else {
      self.in_try = true;
      self.walk_statement(Statement::Block(stmt.block(&self.ast)));
      self.in_try = false;
    }

    if let Some(handler) = stmt.handler(&self.ast) {
      self.walk_catch_clause(handler);
    }

    if let Some(finalizer) = stmt.finalizer(&self.ast) {
      self.walk_statement(Statement::Block(finalizer));
    }
  }

  fn walk_catch_clause(&mut self, catch_clause: CatchClause) {
    self.in_block_scope(|this| {
      if let Some(param) = catch_clause.param(&this.ast) {
        this.enter_pattern(param, |this, ident| {
          this.define_variable(this.ast.get_atom(ident.sym(&this.ast)));
        });
        this.walk_pattern(param)
      }
      let prev = this.prev_statement;
      this.block_pre_walk_statements(catch_clause.body(&this.ast).stmts(&this.ast));
      this.prev_statement = prev;
      this.walk_statement(Statement::Block(catch_clause.body(&this.ast)));
    })
  }

  fn walk_switch_statement(&mut self, stmt: SwitchStmt) {
    self.walk_expression(stmt.discriminant(&self.ast));
    self.walk_switch_cases(stmt.cases(&self.ast));
  }

  fn walk_switch_cases(&mut self, cases: TypedSubRange<SwitchCase>) {
    self.in_block_scope(|this| {
      for case in cases.iter() {
        let case = this.ast.get_node_in_sub_range(case);
        if !case.cons(&this.ast).is_empty() {
          let prev = this.prev_statement;
          this.block_pre_walk_statements(case.cons(&this.ast));
          this.prev_statement = prev;
        }
      }
      for case in cases.iter() {
        let case = this.ast.get_node_in_sub_range(case);
        if let Some(test) = case.test(&this.ast) {
          this.walk_expression(test);
        }
        this.walk_statements(case.cons(&this.ast));
      }
    })
  }

  fn walk_return_statement(&mut self, stmt: ReturnStmt) {
    if let Some(arg) = stmt.arg(&self.ast) {
      self.walk_expression(arg);
    }
  }

  fn walk_throw_stmt(&mut self, stmt: ThrowStmt) {
    self.walk_expression(stmt.arg(&self.ast));
  }

  fn walk_labeled_statement(&mut self, stmt: LabeledStmt) {
    // TODO: self.hooks.label.get
    self.walk_nested_statement(stmt.body(&self.ast));
  }

  fn walk_if_statement(&mut self, stmt: IfStmt) {
    if let Some(result) = self.plugin_drive.clone().statement_if(self, stmt) {
      if result {
        self.walk_nested_statement(stmt.cons(&self.ast));
      } else if let Some(alt) = stmt.alt(&self.ast) {
        self.walk_nested_statement(alt);
      }
    } else {
      self.walk_expression(stmt.test(&self.ast));
      self.walk_nested_statement(stmt.cons(&self.ast));
      if let Some(alt) = stmt.alt(&self.ast) {
        self.walk_nested_statement(alt);
      }
    }
  }

  fn walk_for_statement(&mut self, stmt: ForStmt) {
    self.in_block_scope(|this| {
      if let Some(init) = stmt.init(&this.ast) {
        match init {
          VarDeclOrExpr::VarDecl(decl) => {
            let decl = VariableDeclaration::VarDecl(decl);
            this.block_pre_walk_variable_declaration(decl);
            this.prev_statement = None;
            this.walk_variable_declaration(decl);
          }
          VarDeclOrExpr::Expr(expr) => this.walk_expression(expr),
        }
      }
      if let Some(test) = stmt.test(&this.ast) {
        this.walk_expression(test)
      }
      if let Some(update) = stmt.update(&this.ast) {
        this.walk_expression(update)
      }
      if let Some(body) = stmt.body(&this.ast).as_block() {
        let prev = this.prev_statement;
        this.block_pre_walk_statements(body.stmts(&this.ast));
        this.prev_statement = prev;
        this.walk_statements(body.stmts(&this.ast));
      } else {
        this.walk_nested_statement(stmt.body(&this.ast));
      }
    });
  }

  fn walk_for_of_statement(&mut self, stmt: ForOfStmt) {
    self.in_block_scope(|this| {
      this.walk_for_head(stmt.left(&this.ast));
      this.walk_expression(stmt.right(&this.ast));
      if let Some(body) = stmt.body(&this.ast).as_block() {
        let prev = this.prev_statement;
        this.block_pre_walk_statements(body.stmts(&this.ast));
        this.prev_statement = prev;
        this.walk_statements(body.stmts(&this.ast));
      } else {
        this.walk_nested_statement(stmt.body(&this.ast));
      }
    });
  }

  fn walk_for_in_statement(&mut self, stmt: ForInStmt) {
    self.in_block_scope(|this| {
      this.walk_for_head(stmt.left(&this.ast));
      this.walk_expression(stmt.right(&this.ast));
      if let Some(body) = stmt.body(&this.ast).as_block() {
        let prev = this.prev_statement;
        this.block_pre_walk_statements(body.stmts(&this.ast));
        this.prev_statement = prev;
        this.walk_statements(body.stmts(&this.ast));
      } else {
        this.walk_nested_statement(stmt.body(&this.ast));
      }
    });
  }

  fn walk_for_head(&mut self, for_head: ForHead) {
    match for_head {
      ForHead::VarDecl(decl) => {
        let decl = VariableDeclaration::VarDecl(decl);
        self.block_pre_walk_variable_declaration(decl);
        self.walk_variable_declaration(decl);
      }
      ForHead::UsingDecl(decl) => {
        let decl = VariableDeclaration::UsingDecl(decl);
        self.block_pre_walk_variable_declaration(decl);
        self.walk_variable_declaration(decl);
      }
      ForHead::Pat(pat) => self.walk_pattern(pat),
    }
  }

  fn walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    for declarator in decl.declarators(&self.ast).iter() {
      let declarator = self.ast.get_node_in_sub_range(declarator);
      if let Some(init) = declarator.init(&self.ast)
        && let Some(renamed_identifier) = self.get_rename_identifier(init)
        && let Some(ident) = declarator.name(&self.ast).as_ident()
      {
        let drive = self.plugin_drive.clone();
        if drive
          .can_rename(self, &renamed_identifier)
          .unwrap_or_default()
        {
          if !drive
            .rename(self, init, &renamed_identifier)
            .unwrap_or_default()
          {
            self.set_variable(
              self.ast.get_atom(ident.id(&self.ast).sym(&self.ast)),
              ExportedVariableInfo::Name(renamed_identifier.clone()),
            );
          }
          continue;
        }
      }
      if !self
        .plugin_drive
        .clone()
        .declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.walk_pattern(declarator.name(&self.ast));
        if let Some(init) = declarator.init(&self.ast) {
          self.walk_expression(init);
        }
      }
    }
  }

  fn walk_expression_statement(&mut self, stmt: ExprStmt) {
    self.walk_expression(stmt.expr(&self.ast));
  }

  pub fn walk_expression(&mut self, expr: Expr) {
    match expr {
      Expr::Array(expr) => self.walk_array_expression(expr),
      Expr::Arrow(expr) => self.walk_arrow_function_expression(expr),
      Expr::Assign(expr) => self.walk_assignment_expression(expr),
      Expr::Await(expr) => self.walk_await_expression(expr),
      Expr::Bin(expr) => self.walk_binary_expression(expr),
      Expr::Call(expr) => self.walk_call_expression(expr),
      Expr::Class(expr) => self.walk_class_expression(expr),
      Expr::Cond(expr) => self.walk_conditional_expression(expr),
      Expr::Fn(expr) => self.walk_function_expression(expr),
      Expr::Ident(expr) => self.walk_identifier(expr),
      Expr::MetaProp(expr) => self.walk_meta_property(expr),
      Expr::Member(expr) => self.walk_member_expression(expr),
      Expr::New(expr) => self.walk_new_expression(expr),
      Expr::Object(expr) => self.walk_object_expression(expr),
      Expr::OptChain(expr) => self.walk_chain_expression(expr),
      Expr::Seq(expr) => self.walk_sequence_expression(expr),
      Expr::TaggedTpl(expr) => self.walk_tagged_template_expression(expr),
      Expr::Tpl(expr) => self.walk_template_expression(expr),
      Expr::This(expr) => self.walk_this_expression(expr),
      Expr::Unary(expr) => self.walk_unary_expression(expr),
      Expr::Update(expr) => self.walk_update_expression(expr),
      Expr::Yield(expr) => self.walk_yield_expression(expr),
      Expr::SuperProp(_) | Expr::Lit(_) | Expr::PrivateName(_) | Expr::Invalid(_) => (),
      Expr::JSXMember(_) | Expr::JSXNamespacedName(_) | Expr::JSXEmpty(_) => {
        self.ensure_jsx_enabled();
      }
      Expr::JSXElement(element) => {
        self.ensure_jsx_enabled();
        self.walk_jsx_element(element);
      }
      Expr::JSXFragment(fragment) => {
        self.ensure_jsx_enabled();
        self.walk_jsx_fragment(fragment);
      }
      Expr::Paren(_) => unreachable!(),
    }
  }

  fn walk_yield_expression(&mut self, expr: YieldExpr) {
    if let Some(arg) = expr.arg(&self.ast) {
      self.walk_expression(arg);
    }
  }

  fn walk_update_expression(&mut self, expr: UpdateExpr) {
    self.walk_expression(expr.arg(&self.ast))
  }

  fn walk_unary_expression(&mut self, expr: UnaryExpr) {
    if expr.op(&self.ast) == UnaryOp::TypeOf
      && let Some(expr_info) = self
        .get_member_expression_info_from_expr(expr.arg(&self.ast), AllowedMemberTypes::Expression)
    {
      let MemberExpressionInfo::Expression(expr_info) = expr_info else {
        // we use `AllowedMemberTypes::Expression` above
        unreachable!();
      };
      if expr_info
        .name
        .call_hooks_name(self, |this, for_name| {
          this.plugin_drive.clone().r#typeof(this, expr, for_name)
        })
        .unwrap_or_default()
      {
        return;
      }
    };
    // TODO: expr.arg belongs chain_expression
    self.walk_expression(expr.arg(&self.ast))
  }

  fn walk_this_expression(&mut self, expr: ThisExpr) {
    "this".call_hooks_name(self, |this, for_name| {
      this.plugin_drive.clone().this(this, expr, for_name)
    });
  }

  pub(crate) fn walk_template_expression(&mut self, expr: Tpl) {
    let exprs = expr.exprs(&self.ast);
    self.walk_expressions(exprs);
  }

  fn walk_tagged_template_expression(&mut self, expr: TaggedTpl) {
    self.in_tagged_template_tag = true;
    self.walk_expression(expr.tag(&self.ast));
    self.in_tagged_template_tag = false;

    let exprs = expr.tpl(&self.ast).exprs(&self.ast);
    self.walk_expressions(exprs);
  }

  fn walk_sequence_expression(&mut self, expr: SeqExpr) {
    let exprs = expr.exprs(&self.ast);
    if self.is_statement_level_expression(expr.span(&self.ast))
      && let Some(old) = self.statement_path.pop()
    {
      let prev = self.prev_statement;
      for expr in exprs.iter() {
        let expr = self.ast.get_node_in_sub_range(expr);
        self.statement_path.push(expr.span(&self.ast).into());
        self.walk_expression(expr);
        self.prev_statement = self.statement_path.pop();
      }
      self.prev_statement = prev;
      self.statement_path.push(old);
    } else {
      self.walk_expressions(exprs);
    }
  }

  fn ensure_jsx_enabled(&self) {
    if !self.javascript_options.jsx.unwrap_or_default() {
      unreachable!();
    }
  }

  fn walk_jsx_element(&mut self, element: JSXElement) {
    self.walk_jsx_element_name(element.opening(&self.ast).name(&self.ast));
    for attr in element.opening(&self.ast).attrs(&self.ast).iter() {
      let attr = self.ast.get_node_in_sub_range(attr);
      self.walk_jsx_attr_or_spread(attr);
    }
    for child in element.children(&self.ast).iter() {
      let child = self.ast.get_node_in_sub_range(child);
      self.walk_jsx_child(child);
    }
    if let Some(closing) = element.closing(&self.ast) {
      self.walk_jsx_element_name(closing.name(&self.ast));
    }
  }

  fn walk_jsx_fragment(&mut self, fragment: JSXFragment) {
    for child in fragment.children(&self.ast).iter() {
      let child = self.ast.get_node_in_sub_range(child);
      self.walk_jsx_child(child);
    }
  }

  fn walk_jsx_child(&mut self, child: JSXElementChild) {
    match child {
      JSXElementChild::JSXElement(element) => self.walk_jsx_element(element),
      JSXElementChild::JSXFragment(fragment) => self.walk_jsx_fragment(fragment),
      JSXElementChild::JSXExprContainer(container) => self.walk_jsx_expr_container(container),
      JSXElementChild::JSXSpreadChild(spread) => self.walk_expression(spread.expr(&self.ast)),
      JSXElementChild::JSXText(_) => (),
    }
  }

  fn walk_jsx_expr_container(&mut self, container: JSXExprContainer) {
    match container.expr(&self.ast) {
      JSXExpr::Expr(expr) => self.walk_expression(expr),
      JSXExpr::JSXEmptyExpr(_) => (),
    }
  }

  fn walk_jsx_attr_or_spread(&mut self, attr: JSXAttrOrSpread) {
    match attr {
      JSXAttrOrSpread::JSXAttr(attr) => self.walk_jsx_attr(attr),
      JSXAttrOrSpread::SpreadElement(spread) => self.walk_expression(spread.expr(&self.ast)),
    }
  }

  fn walk_jsx_attr(&mut self, attr: JSXAttr) {
    if let Some(value) = attr.value(&self.ast) {
      self.walk_jsx_attr_value(value);
    }
  }

  fn walk_jsx_attr_value(&mut self, value: JSXAttrValue) {
    match value {
      JSXAttrValue::Str(_) => (),
      JSXAttrValue::JSXExprContainer(container) => self.walk_jsx_expr_container(container),
      JSXAttrValue::JSXElement(element) => self.walk_jsx_element(element),
      JSXAttrValue::JSXFragment(fragment) => self.walk_jsx_fragment(fragment),
    }
  }

  fn walk_jsx_element_name(&mut self, name: JSXElementName) {
    match name {
      JSXElementName::Ident(ident) => self.walk_identifier(ident),
      JSXElementName::JSXMemberExpr(member) => self.walk_jsx_member_expr(member),
      JSXElementName::JSXNamespacedName(namespaced) => self.walk_jsx_namespaced_name(namespaced),
    }
  }

  fn walk_jsx_member_expr(&mut self, member: JSXMemberExpr) {
    let member_expr = self.jsx_member_expr_to_member_expr(member);
    self.walk_member_expression(member_expr);
  }

  fn jsx_member_expr_to_member_expr(&mut self, member: JSXMemberExpr) -> MemberExpr {
    let obj = self.jsx_object_to_expr(member.obj(&self.ast));
    self.ast.member_expr(
      member.span(&self.ast),
      obj,
      MemberProp::Ident(member.prop(&self.ast)),
    )
  }

  fn jsx_object_to_expr(&mut self, obj: JSXObject) -> Expr {
    match obj {
      JSXObject::Ident(ident) => Expr::Ident(ident),
      JSXObject::JSXMemberExpr(member) => Expr::Member(self.jsx_member_expr_to_member_expr(member)),
    }
  }

  fn walk_jsx_namespaced_name(&mut self, name: JSXNamespacedName) {
    self.walk_ident_name(name.ns(&self.ast));
    self.walk_ident_name(name.name(&self.ast));
  }

  fn walk_ident_name(&mut self, name: IdentName) {
    let ident = self
      .ast
      .ident(name.span(&self.ast), name.sym(&self.ast), false);
    self.walk_identifier(ident);
  }

  fn walk_object_expression(&mut self, expr: ObjectLit) {
    for prop in expr.props(&self.ast).iter() {
      let prop = self.ast.get_node_in_sub_range(prop);
      self.walk_property_or_spread(prop);
    }
  }

  fn walk_property_or_spread(&mut self, prop: PropOrSpread) {
    match prop {
      PropOrSpread::SpreadElement(spread) => self.walk_expression(spread.expr(&self.ast)),
      PropOrSpread::Prop(prop) => self.walk_property(prop),
    }
  }

  fn walk_key_value_prop(&mut self, kv: KeyValueProp) {
    if kv.key(&self.ast).is_computed() {
      // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_prop_name`
      self.walk_prop_name(kv.key(&self.ast));
    }
    self.walk_expression(kv.value(&self.ast));
  }

  fn walk_getter_prop(&mut self, getter: GetterProp) {
    self.walk_prop_name(getter.key(&self.ast));
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    self.in_function_scope(true, std::iter::empty(), |parser| {
      if let Some(body) = getter.body(&parser.ast) {
        parser.detect_mode(body.stmts(&parser.ast));
        let prev = parser.prev_statement;
        parser.pre_walk_statement(Statement::Block(body));
        parser.prev_statement = prev;
        parser.walk_statement(Statement::Block(body));
      }
    });
    self.top_level_scope = was_top_level;
  }

  fn walk_setter_prop(&mut self, setter: SetterProp) {
    self.walk_prop_name(setter.key(&self.ast));
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    self.in_function_scope(true, std::iter::once(setter.param(&self.ast)), |parser| {
      if let Some(body) = setter.body(&parser.ast) {
        parser.detect_mode(body.stmts(&parser.ast));
        let prev = parser.prev_statement;
        parser.pre_walk_statement(Statement::Block(body));
        parser.prev_statement = prev;
        parser.walk_statement(Statement::Block(body));
      }
    });
    self.top_level_scope = was_top_level;
  }

  fn walk_property(&mut self, prop: Prop) {
    match prop {
      Prop::Shorthand(ident) => {
        self.in_short_hand = true;
        self.walk_identifier(ident);
        self.in_short_hand = false;
      }
      Prop::KeyValue(kv) => self.walk_key_value_prop(kv),
      Prop::Assign(assign) => self.walk_expression(assign.value(&self.ast)),
      Prop::Getter(getter) => self.walk_getter_prop(getter),
      Prop::Setter(setter) => self.walk_setter_prop(setter),
      Prop::Method(method) => {
        self.walk_prop_name(method.key(&self.ast));
        let was_top_level = self.top_level_scope;
        self.top_level_scope = TopLevelScope::False;
        let params = method
          .function(&self.ast)
          .params(&self.ast)
          .iter()
          .map(|p| self.ast.get_node_in_sub_range(p).pat(&self.ast))
          .collect_vec();
        self.in_function_scope(true, params.into_iter(), |this| {
          this.walk_function(method.function(&this.ast));
        });
        self.top_level_scope = was_top_level;
      }
    }
  }

  fn walk_prop_name(&mut self, prop_name: PropName) {
    if let Some(computed) = prop_name.as_computed() {
      self.walk_expression(computed.expr(&self.ast));
    }
  }

  fn walk_new_expression(&mut self, expr: NewExpr) {
    if let Some(MemberExpressionInfo::Expression(info)) = self
      .get_member_expression_info_from_expr(expr.callee(&self.ast), AllowedMemberTypes::Expression)
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
    self.walk_expression(expr.callee(&self.ast));
    if let Some(args) = expr.args(&self.ast) {
      self.walk_expr_or_spread(args);
    }
  }

  fn walk_meta_property(&mut self, expr: MetaPropExpr) {
    let Some(root_name) = expr.get_root_name(&self.ast) else {
      unreachable!()
    };
    self
      .plugin_drive
      .clone()
      .meta_property(self, &root_name, expr.span(&self.ast));
  }

  fn walk_conditional_expression(&mut self, expr: CondExpr) {
    let result = self
      .plugin_drive
      .clone()
      .expression_conditional_operation(self, expr);

    if let Some(result) = result {
      if result {
        self.walk_expression(expr.cons(&self.ast));
      } else {
        self.walk_expression(expr.alt(&self.ast));
      }
    } else {
      self.walk_expression(expr.test(&self.ast));
      self.walk_expression(expr.cons(&self.ast));
      self.walk_expression(expr.alt(&self.ast));
    }
  }

  fn walk_class_expression(&mut self, expr: ClassExpr) {
    self.walk_class(expr.class(&self.ast), ClassDeclOrExpr::Expr(expr));
  }

  fn walk_chain_expression(&mut self, expr: OptChainExpr) {
    if self
      .plugin_drive
      .clone()
      .optional_chaining(self, expr)
      .is_none()
    {
      self.enter_optional_chain(
        expr,
        |parser, call| parser.walk_opt_call(call),
        |parser, member| parser.walk_member_expression(member),
      );
    }
  }

  fn walk_member_expression(&mut self, expr: MemberExpr) {
    // println!("{:#?}", expr);
    if let Some(expr_info) =
      self.get_member_expression_info(Expr::Member(expr), AllowedMemberTypes::all())
    {
      match expr_info {
        MemberExpressionInfo::Expression(expr_info) => {
          let drive = self.plugin_drive.clone();
          if expr_info
            .name
            .call_hooks_name(self, |this, for_name| drive.member(this, expr, for_name))
            .unwrap_or_default()
          {
            return;
          }
          if expr_info
            .root_info
            .call_hooks_name(self, |this, for_name| {
              drive.member_chain(
                this,
                expr,
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
            Some(|this: &mut Self| {
              drive.unhandled_expression_member_chain(this, &expr_info.root_info, expr)
            }),
          );
          return;
        }
        MemberExpressionInfo::Call(expr_info) => {
          if expr_info
            .root_info
            .call_hooks_name(self, |this, for_name| {
              this.plugin_drive.clone().member_chain_of_call_member_chain(
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
    self.member_expr_in_optional_chain = false;
    self.walk_expression(expr.obj(&self.ast));
    if let MemberProp::Computed(computed) = expr.prop(&self.ast) {
      self.walk_expression(computed.expr(&self.ast))
    }
  }

  fn walk_member_expression_with_expression_name<F>(
    &mut self,
    expr: MemberExpr,
    name: &str,
    on_unhandled: Option<F>,
  ) where
    F: FnOnce(&mut Self) -> Option<bool>,
  {
    if let Some(member) = expr.obj(&self.ast).as_member()
      && let Some(len) = member_prop_len(&self.ast, expr.prop(&self.ast))
    {
      let origin = name.len();
      let name = &name[0..origin - 1 - len];
      if name
        .call_hooks_name(self, |this, for_name| {
          this.plugin_drive.clone().member(this, member, for_name)
        })
        .unwrap_or_default()
      {
        return;
      }
      self.walk_member_expression_with_expression_name(member, name, on_unhandled);
    } else if on_unhandled.is_none() {
      self.walk_expression(expr.obj(&self.ast));
    } else if let Some(on_unhandled) = on_unhandled
      && !on_unhandled(self).unwrap_or_default()
    {
      self.walk_expression(expr.obj(&self.ast));
    }

    if let MemberProp::Computed(computed) = expr.prop(&self.ast) {
      self.walk_expression(computed.expr(&self.ast))
    }
  }

  fn walk_opt_call(&mut self, expr: OptCall) {
    // TODO: remove clone
    let call_expr = self.ast.call_expr(
      expr.span(&self.ast),
      Callee::Expr(expr.callee(&self.ast)),
      expr.args(&self.ast),
    );
    self.walk_call_expression(call_expr);
  }

  /// Walk IIFE function
  ///
  /// # Panics
  /// Either `Params` of `expr` or `params` passed in should be `BindingIdent`.
  fn _walk_iife(
    &mut self,
    expr: Expr,
    args: impl Iterator<Item = Expr>,
    current_this: Option<Expr>,
  ) {
    fn get_var_name(parser: &mut JavascriptParser, expr: Expr) -> Option<ExportedVariableInfo> {
      if let Some(rename_identifier) = parser.get_rename_identifier(expr)
        && let drive = parser.plugin_drive.clone()
        && rename_identifier
          .call_hooks_name(parser, |this, for_name| drive.can_rename(this, for_name))
          .unwrap_or_default()
        && !rename_identifier
          .call_hooks_name(parser, |this, for_name| drive.rename(this, expr, for_name))
          .unwrap_or_default()
      {
        let variable = parser
          .get_variable_info(&rename_identifier)
          .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
          .unwrap_or(ExportedVariableInfo::Name(rename_identifier));
        return Some(variable);
      }
      parser.walk_expression(expr);
      None
    }

    let rename_this = current_this.and_then(|this| get_var_name(self, this));
    let variable_info_for_args = args
      .map(|param| get_var_name(self, param))
      .collect::<Vec<_>>();

    let mut params = vec![];
    let mut scope_params = vec![];
    if let Some(fn_expr) = expr.as_fn() {
      for (i, pat) in fn_expr
        .function(&self.ast)
        .params(&self.ast)
        .iter()
        .enumerate()
      {
        let pat = self.ast.get_node_in_sub_range(pat).pat(&self.ast);
        // SAFETY: is_simple_function will ensure pat is always a BindingIdent.
        let ident = pat.as_ident().expect("should be a `BindingIdent`");
        params.push(ident);
        if variable_info_for_args.get(i).is_none() {
          scope_params.push(pat);
        }
      }
    } else if let Some(arrow_expr) = expr.as_arrow() {
      for (i, pat) in arrow_expr.params(&self.ast).iter().enumerate() {
        let pat = self.ast.get_node_in_sub_range(pat);
        // SAFETY: is_simple_function will ensure pat is always a BindingIdent.
        let ident = pat.as_ident().expect("should be a `BindingIdent`");
        params.push(ident);
        if variable_info_for_args.get(i).is_none() {
          scope_params.push(pat);
        }
      }
    }

    // Add function name in scope for recursive calls
    if let Some(function) = expr.as_fn()
      && let Some(ident) = function.ident(&self.ast)
    {
      scope_params.push(warp_ident_to_pat(&mut self.ast, ident));
    }

    let was_top_level_scope = self.top_level_scope;
    self.top_level_scope =
      if !matches!(was_top_level_scope, TopLevelScope::False) && expr.is_arrow() {
        TopLevelScope::ArrowFunction
      } else {
        TopLevelScope::False
      };

    self.in_function_scope(true, scope_params.into_iter(), |parser| {
      if let Some(this) = rename_this
        && !expr.is_arrow()
      {
        parser.set_variable("this".into(), this)
      }
      for (i, var_info) in variable_info_for_args.into_iter().enumerate() {
        if let Some(var_info) = var_info
          && let Some(param) = params.get(i)
        {
          let param = parser.ast.get_atom(param.id(&parser.ast).sym(&parser.ast));
          parser.set_variable(param, var_info);
        }
      }

      if let Some(expr) = expr.as_fn() {
        if let Some(stmt) = expr.function(&parser.ast).body(&parser.ast) {
          parser.detect_mode(stmt.stmts(&parser.ast));
          let prev = parser.prev_statement;
          parser.pre_walk_statement(Statement::Block(stmt));
          parser.prev_statement = prev;
          parser.walk_statement(Statement::Block(stmt));
        }
      } else if let Some(expr) = expr.as_arrow() {
        match expr.body(&parser.ast) {
          BlockStmtOrExpr::BlockStmt(stmt) => {
            parser.detect_mode(stmt.stmts(&parser.ast));
            let prev = parser.prev_statement;
            parser.pre_walk_statement(Statement::Block(stmt));
            parser.prev_statement = prev;
            parser.walk_statement(Statement::Block(stmt));
          }
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        }
      }
    });
    self.top_level_scope = was_top_level_scope;
  }

  fn walk_call_expression(&mut self, expr: CallExpr) {
    fn is_simple_function(ast: &Ast, params: TypedSubRange<Param>) -> bool {
      params.iter().all(|p| {
        let p = ast.get_node_in_sub_range(p);
        matches!(p.pat(ast), Pat::Ident(_))
      })
    }

    // FIXME: should align to webpack
    match expr.callee(&self.ast) {
      Callee::Expr(callee) => {
        if let Expr::Member(member_expr) = callee
          && let Expr::Fn(fn_expr) = member_expr.obj(&self.ast)
          && let MemberProp::Ident(ident) = member_expr.prop(&self.ast)
          && (self.ast.get_utf8(ident.sym(&self.ast)) == "call"
            || self.ast.get_utf8(ident.sym(&self.ast)) == "bind")
          && !expr.args(&self.ast).is_empty()
          && is_simple_function(&self.ast, fn_expr.function(&self.ast).params(&self.ast))
        {
          // (function(…) { }).call(…)
          let mut params = expr
            .args(&self.ast)
            .iter()
            .map(|arg| self.ast.get_node_in_sub_range(arg).expr(&self.ast))
            .collect_vec()
            .into_iter();
          let this = params.next();
          self._walk_iife(member_expr.obj(&self.ast), params, this)
        } else if let Expr::Member(member_expr) = callee
          && let Expr::Fn(fn_expr) = member_expr.obj(&self.ast)
          && let MemberProp::Ident(ident) = member_expr.prop(&self.ast)
          && (self.ast.get_utf8(ident.sym(&self.ast)) == "call"
            || self.ast.get_utf8(ident.sym(&self.ast)) == "bind")
          && !expr.args(&self.ast).is_empty()
          && is_simple_function(&self.ast, fn_expr.function(&self.ast).params(&self.ast))
        {
          // (function(…) { }.call(…))
          let mut params = expr
            .args(&self.ast)
            .iter()
            .map(|arg| self.ast.get_node_in_sub_range(arg).expr(&self.ast))
            .collect_vec()
            .into_iter();
          let this = params.next();
          self._walk_iife(member_expr.obj(&self.ast), params, this)
        } else if let Expr::Fn(fn_expr) = callee
          && is_simple_function(&self.ast, fn_expr.function(&self.ast).params(&self.ast))
        {
          // (function(…) { })(…)
          self._walk_iife(
            callee,
            expr
              .args(&self.ast)
              .iter()
              .map(|arg| self.ast.get_node_in_sub_range(arg).expr(&self.ast))
              .collect_vec()
              .into_iter(),
            None,
          )
        } else if let Expr::Fn(fn_expr) = callee
          && is_simple_function(&self.ast, fn_expr.function(&self.ast).params(&self.ast))
        {
          // ((…) => { }(…))
          self._walk_iife(
            callee,
            expr
              .args(&self.ast)
              .iter()
              .map(|arg| self.ast.get_node_in_sub_range(arg).expr(&self.ast))
              .collect_vec()
              .into_iter(),
            None,
          )
        } else if let Expr::Arrow(arrow_expr) = callee
          && arrow_expr
            .params(&self.ast)
            .iter()
            .all(|p| self.ast.get_node_in_sub_range(p).as_ident().is_some())
        {
          // (function(…) { }(…))
          self._walk_iife(
            callee,
            expr
              .args(&self.ast)
              .iter()
              .map(|arg| self.ast.get_node_in_sub_range(arg).expr(&self.ast))
              .collect_vec()
              .into_iter(),
            None,
          )
        } else {
          if let Expr::Member(member) = callee {
            if let Some(MemberExpressionInfo::Call(expr_info)) = self
              .get_member_expression_info(Expr::Member(member), AllowedMemberTypes::CallExpression)
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
            if let Some(call) = member.obj(&self.ast).as_call()
              && call.callee(&self.ast).is_import()
              && let Some(prop) = member.prop(&self.ast).as_ident()
              && self.ast.get_utf8(prop.sym(&self.ast)) == "then"
            {
              // import(…).then(…)
              if self
                .plugin_drive
                .clone()
                .import_call(self, call, Some(expr))
                .unwrap_or_default()
              {
                return;
              }
            }
          }
          let evaluated_callee = self.evaluate_expression(callee);
          if evaluated_callee.is_identifier() {
            let members = evaluated_callee
              .members()
              .map_or_else(|| Cow::Owned(Vec::new()), Cow::Borrowed);
            let members_optionals = evaluated_callee.members_optionals().map_or_else(
              || Cow::Owned(members.iter().map(|_| false).collect::<Vec<_>>()),
              Cow::Borrowed,
            );
            let member_ranges = evaluated_callee
              .member_ranges()
              .map_or_else(|| Cow::Owned(Vec::new()), Cow::Borrowed);
            let drive = self.plugin_drive.clone();
            if evaluated_callee
              .root_info()
              .call_hooks_name(self, |parser, for_name| {
                drive.call_member_chain(
                  parser,
                  expr,
                  for_name,
                  &members,
                  &members_optionals,
                  &member_ranges,
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

          if let Some(member) = callee.as_member() {
            self.walk_expression(member.obj(&self.ast));
            if let Some(computed) = member.prop(&self.ast).as_computed() {
              self.walk_expression(computed.expr(&self.ast));
            }
          } else if let Some(member) = callee.as_super_prop() {
            if let Some(computed) = member.prop(&self.ast).as_computed() {
              self.walk_expression(computed.expr(&self.ast));
            }
          } else {
            self.walk_expression(callee);
          }
          self.walk_expr_or_spread(expr.args(&self.ast));
        }
      }
      Callee::Import(_) => {
        // In webpack this is walkImportExpression, import() is a ImportExpression instead of CallExpression with Callee::Import
        if self
          .plugin_drive
          .clone()
          .import_call(self, expr, None)
          .unwrap_or_default()
        {
          return;
        }

        self.walk_expr_or_spread(expr.args(&self.ast));
      }
      Callee::Super(_) => {
        // Do nothing about super, same as webpack
        self.walk_expr_or_spread(expr.args(&self.ast));
      }
    }
  }

  pub fn walk_expr_or_spread(&mut self, args: TypedSubRange<ExprOrSpread>) {
    for arg in args.iter() {
      let arg = self.ast.get_node_in_sub_range(arg);
      self.walk_expression(arg.expr(&self.ast));
    }
  }

  fn walk_left_right_expression(&mut self, expr: BinExpr) {
    self.walk_expression(expr.left(&self.ast));
    self.walk_expression(expr.right(&self.ast));
  }

  fn walk_binary_expression(&mut self, expr: BinExpr) {
    if is_logic_op(expr.op(&self.ast)) {
      if let Some(keep_right) = self
        .plugin_drive
        .clone()
        .expression_logical_operator(self, expr)
      {
        if keep_right {
          self.walk_expression(expr.right(&self.ast));
        }
      } else {
        self.walk_left_right_expression(expr)
      }
    } else if self
      .plugin_drive
      .clone()
      .binary_expression(self, expr)
      .is_none()
    {
      self.walk_left_right_expression(expr)
    }
  }

  fn walk_await_expression(&mut self, expr: AwaitExpr) {
    if self.is_top_level_scope() {
      self.plugin_drive.clone().top_level_await_expr(self, expr);
    }
    self.walk_expression(expr.arg(&self.ast));
  }

  fn walk_identifier(&mut self, identifier: Ident) {
    let identifier_sym = self.ast.get_atom(identifier.sym(&self.ast));
    identifier_sym.call_hooks_name(self, |this, for_name| {
      this
        .plugin_drive
        .clone()
        .identifier(this, identifier, for_name)
    });
  }

  fn get_rename_identifier(&mut self, expr: Expr) -> Option<Atom> {
    let result = self.evaluate_expression(expr);
    result.is_identifier().then(|| result.identifier().clone())
  }

  fn walk_assignment_expression(&mut self, expr: AssignExpr) {
    if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = expr.left(&self.ast) {
      let ident = ident.id(&self.ast);
      if let Some(rename_identifier) = self.get_rename_identifier(expr.right(&self.ast))
        && let drive = self.plugin_drive.clone()
        && rename_identifier
          .call_hooks_name(self, |this, for_name| drive.can_rename(this, for_name))
          .unwrap_or_default()
      {
        if !rename_identifier
          .call_hooks_name(self, |this, for_name| {
            drive.rename(this, expr.right(&this.ast), for_name)
          })
          .unwrap_or_default()
        {
          let variable = self
            .get_variable_info(&rename_identifier)
            .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
            .unwrap_or(ExportedVariableInfo::Name(rename_identifier));
          self.set_variable(self.ast.get_atom(ident.sym(&self.ast)), variable);
        }
        return;
      }
      self.walk_expression(expr.right(&self.ast));
      let pat = warp_ident_to_pat(&mut self.ast, ident);
      self.enter_pattern(pat, |this, ident| {
        let ident_sym = this.ast.get_atom(ident.sym(&this.ast));
        if !ident_sym
          .call_hooks_name(this, |this, for_name| {
            this.plugin_drive.clone().assign(this, expr, for_name)
          })
          .unwrap_or_default()
        {
          // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_identifier`
          this.walk_identifier(ident);
        }
      });
    } else if let Some(pat) = expr.left(&self.ast).as_pat() {
      self.walk_expression(expr.right(&self.ast));
      self.enter_assign_target_pattern(pat, |this: &mut JavascriptParser<'_>, ident| {
        let ident = this.ast.get_atom(ident.sym(&this.ast));
        if !ident
          .call_hooks_name(this, |this, for_name| {
            this.plugin_drive.clone().assign(this, expr, for_name)
          })
          .unwrap_or_default()
        {
          this.define_variable(ident);
        }
      });
      self.walk_assign_target_pattern(pat);
    } else if let Some(SimpleAssignTarget::Member(member)) = expr.left(&self.ast).as_simple() {
      if let Some(MemberExpressionInfo::Expression(expr_name)) =
        self.get_member_expression_info(Expr::Member(member), AllowedMemberTypes::Expression)
        && expr_name
          .root_info
          .call_hooks_name(self, |parser, for_name| {
            parser.plugin_drive.clone().assign_member_chain(
              parser,
              expr,
              &expr_name.members,
              for_name,
            )
          })
          .unwrap_or_default()
      {
        return;
      }
      self.walk_expression(expr.right(&self.ast));
      match expr.left(&self.ast) {
        AssignTarget::Simple(simple) => self.walk_simple_assign_target(simple),
        AssignTarget::Pat(pat) => self.walk_assign_target_pattern(pat),
      }
    } else {
      self.walk_expression(expr.right(&self.ast));
      match expr.left(&self.ast) {
        AssignTarget::Simple(simple) => self.walk_simple_assign_target(simple),
        AssignTarget::Pat(pat) => self.walk_assign_target_pattern(pat),
      }
    }
    // TODO:
    // else if let Some(member) = expr.left.as_expr().and_then(|expr| expr.as_member()) {
    // }
  }

  fn walk_arrow_function_expression(&mut self, expr: ArrowExpr) {
    let was_top_level_scope = self.top_level_scope;
    if !matches!(was_top_level_scope, TopLevelScope::False) {
      self.top_level_scope = TopLevelScope::ArrowFunction;
    }
    let params = expr
      .params(&self.ast)
      .iter()
      .map(|p| self.ast.get_node_in_sub_range(p))
      .collect_vec();
    self.in_function_scope(false, params.into_iter(), |this| {
      for param in expr.params(&this.ast).iter() {
        let param = this.ast.get_node_in_sub_range(param);
        this.walk_pattern(param)
      }
      match expr.body(&this.ast) {
        BlockStmtOrExpr::BlockStmt(stmt) => {
          this.detect_mode(stmt.stmts(&this.ast));
          let prev = this.prev_statement;
          this.pre_walk_statement(Statement::Block(stmt));
          this.prev_statement = prev;
          this.walk_statement(Statement::Block(stmt));
        }
        BlockStmtOrExpr::Expr(expr) => this.walk_expression(expr),
      }
    });
    self.top_level_scope = was_top_level_scope;
  }

  fn walk_expressions(&mut self, expressions: TypedSubRange<Expr>) {
    for expr in expressions.iter() {
      let expr = self.ast.get_node_in_sub_range(expr);
      self.walk_expression(expr)
    }
  }

  fn walk_array_expression(&mut self, expr: ArrayLit) {
    for elem in expr.elems(&self.ast).iter() {
      let elem = self.ast.get_node_in_sub_range(elem);
      if let Some(elem) = elem {
        self.walk_expression(elem.expr(&self.ast))
      }
    }
  }

  fn walk_nested_statement(&mut self, stmt: Stmt) {
    self.prev_statement = None;
    self.walk_statement(Statement::from_stmt(stmt, &self.ast));
  }

  fn walk_do_while_statement(&mut self, stmt: DoWhileStmt) {
    self.walk_nested_statement(stmt.body(&self.ast));
    self.walk_expression(stmt.test(&self.ast));
  }

  fn walk_block_statement(&mut self, stmt: BlockStmt) {
    self.in_block_scope(|this| {
      let prev = this.prev_statement;
      this.block_pre_walk_statements(stmt.stmts(&this.ast));
      this.prev_statement = prev;
      this.walk_statements(stmt.stmts(&this.ast));
    })
  }

  fn walk_function_declaration(&mut self, decl: MaybeNamedFunctionDecl) {
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    let params = decl
      .function()
      .params(&self.ast)
      .iter()
      .map(|param| self.ast.get_node_in_sub_range(param).pat(&self.ast))
      .collect_vec();
    self.in_function_scope(true, params.into_iter(), |this| {
      this.walk_function(decl.function());
    });
    self.top_level_scope = was_top_level;
  }

  fn walk_function(&mut self, f: Function) {
    for param in f.params(&self.ast).iter() {
      let param = self.ast.get_node_in_sub_range(param);
      self.walk_pattern(param.pat(&self.ast))
    }
    if let Some(body) = f.body(&self.ast) {
      self.detect_mode(body.stmts(&self.ast));
      let prev = self.prev_statement;
      self.pre_walk_statement(Statement::Block(body));
      self.prev_statement = prev;
      self.walk_statement(Statement::Block(body));
    }
  }

  fn walk_function_expression(&mut self, expr: FnExpr) {
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
    let mut scope_params: Vec<_> = expr
      .function(&self.ast)
      .params(&self.ast)
      .iter()
      .map(|params| self.ast.get_node_in_sub_range(params).pat(&self.ast))
      .collect();

    if let Some(pat) = expr
      .ident(&self.ast)
      .map(|ident| warp_ident_to_pat(&mut self.ast, ident))
    {
      scope_params.push(pat);
    }

    self.in_function_scope(true, scope_params.into_iter(), |this| {
      this.walk_function(expr.function(&this.ast));
    });
    self.top_level_scope = was_top_level;
  }

  pub fn walk_pattern(&mut self, pat: Pat) {
    match pat {
      Pat::Array(array) => self.walk_array_pattern(array),
      Pat::Assign(assign) => self.walk_assignment_pattern(assign),
      Pat::Object(obj) => self.walk_object_pattern(obj),
      Pat::Rest(rest) => self.walk_rest_element(rest),
      Pat::Expr(expr) => self.walk_expression(expr),
      Pat::Ident(_) => (),
      Pat::Invalid(_) => (),
    }
  }

  fn walk_simple_assign_target(&mut self, target: SimpleAssignTarget) {
    match target {
      SimpleAssignTarget::Ident(ident) => self.walk_identifier(ident.id(&self.ast)),
      SimpleAssignTarget::Member(member) => self.walk_member_expression(member),
      SimpleAssignTarget::OptChain(expr) => self.walk_chain_expression(expr),
      SimpleAssignTarget::SuperProp(_) => (),
      SimpleAssignTarget::Paren(_) | SimpleAssignTarget::Invalid(_) => unreachable!(),
    }
  }

  fn walk_assign_target_pattern(&mut self, pat: AssignTargetPat) {
    match pat {
      AssignTargetPat::Array(array) => self.walk_array_pattern(array),
      AssignTargetPat::Object(obj) => self.walk_object_pattern(obj),
      AssignTargetPat::Invalid(_) => (),
    }
  }

  fn walk_rest_element(&mut self, rest: RestPat) {
    self.walk_pattern(rest.arg(&self.ast));
  }

  fn walk_object_pattern(&mut self, obj: ObjectPat) {
    for prop in obj.props(&self.ast).iter() {
      let prop = self.ast.get_node_in_sub_range(prop);
      match prop {
        ObjectPatProp::KeyValue(kv) => {
          if kv.key(&self.ast).is_computed() {
            // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_prop_name`
            self.walk_prop_name(kv.key(&self.ast));
          }
          self.walk_pattern(kv.value(&self.ast));
        }
        ObjectPatProp::Assign(assign) => {
          if let Some(value) = assign.value(&self.ast) {
            self.walk_expression(value);
          }
        }
        ObjectPatProp::Rest(rest) => self.walk_rest_element(rest),
      }
    }
  }

  fn walk_assignment_pattern(&mut self, pat: AssignPat) {
    self.walk_expression(pat.right(&self.ast));
    self.walk_pattern(pat.left(&self.ast));
  }

  fn walk_array_pattern(&mut self, pat: ArrayPat) {
    for elem in pat.elems(&self.ast).iter() {
      let elem = self.ast.get_node_in_sub_range(elem);
      if let Some(elem) = elem {
        self.walk_pattern(elem);
      }
    }
  }

  fn walk_class_declaration(&mut self, decl: MaybeNamedClassDecl) {
    self.walk_class(decl.class(), ClassDeclOrExpr::Decl(decl));
  }

  fn walk_class(&mut self, classy: Class, class_decl_or_expr: ClassDeclOrExpr) {
    if let Some(super_class) = classy.super_class(&self.ast)
      && !self
        .plugin_drive
        .clone()
        .class_extends_expression(self, super_class, class_decl_or_expr)
        .unwrap_or_default()
    {
      self.walk_expression(super_class);
    }

    // TODO: define variable for class expression in block pre walk
    let scope_params = if let ClassDeclOrExpr::Expr(class_expr) = class_decl_or_expr
      && let Some(pat) = class_expr
        .ident(&self.ast)
        .map(|ident| warp_ident_to_pat(&mut self.ast, ident))
    {
      vec![pat]
    } else {
      vec![]
    };

    self.in_class_scope(true, scope_params.into_iter(), |this| {
      for class_element in classy.body(&this.ast).iter() {
        let class_element = this.ast.get_node_in_sub_range(class_element);
        if this
          .plugin_drive
          .clone()
          .class_body_element(this, class_element, class_decl_or_expr)
          .unwrap_or_default()
        {
          continue;
        }

        match class_element {
          ClassMember::Constructor(ctor) => {
            if ctor.key(&this.ast).is_computed() {
              // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_prop_name`
              this.walk_prop_name(ctor.key(&this.ast));
            }

            if this
              .plugin_drive
              .clone()
              .class_body_value(
                this,
                class_element,
                ctor.span(&this.ast),
                class_decl_or_expr,
              )
              .unwrap_or_default()
            {
              continue;
            }

            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;

            let params = ctor
              .params(&this.ast)
              .iter()
              .map(|p| {
                let p = this.ast.get_node_in_sub_range(p);
                let p = p.as_param().expect("should only contain param");
                p.pat(&this.ast)
              })
              .collect_vec();
            this.in_function_scope(true, params.clone().into_iter(), |this| {
              for param in params {
                this.walk_pattern(param)
              }

              if let Some(body) = ctor.body(&this.ast) {
                this.detect_mode(body.stmts(&this.ast));
                let prev = this.prev_statement;
                this.pre_walk_statement(Statement::Block(body));
                this.prev_statement = prev;
                this.walk_statement(Statement::Block(body));
              }
            });

            this.top_level_scope = was_top_level;
          }
          ClassMember::Method(method) => {
            if method.key(&this.ast).is_computed() {
              // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_prop_name`
              this.walk_prop_name(method.key(&this.ast));
            }

            if this
              .plugin_drive
              .clone()
              .class_body_value(
                this,
                class_element,
                method.span(&this.ast),
                class_decl_or_expr,
              )
              .unwrap_or_default()
            {
              continue;
            }

            let was_top_level = this.top_level_scope;
            let params = method
              .function(&this.ast)
              .params(&this.ast)
              .iter()
              .map(|p| this.ast.get_node_in_sub_range(p).pat(&this.ast))
              .collect_vec();
            this.top_level_scope = TopLevelScope::False;
            this.in_function_scope(true, params.into_iter(), |this| {
              this.walk_function(method.function(&this.ast))
            });
            this.top_level_scope = was_top_level;
          }
          ClassMember::PrivateMethod(method) => {
            if this
              .plugin_drive
              .clone()
              .class_body_value(
                this,
                class_element,
                method.span(&this.ast),
                class_decl_or_expr,
              )
              .unwrap_or_default()
            {
              continue;
            }
            // method.key is always not computed in private method, so we don't need to walk it
            let was_top_level = this.top_level_scope;
            let params = method
              .function(&this.ast)
              .params(&this.ast)
              .iter()
              .map(|p| this.ast.get_node_in_sub_range(p).pat(&this.ast))
              .collect_vec();
            this.top_level_scope = TopLevelScope::False;
            this.in_function_scope(true, params.into_iter(), |this| {
              this.walk_function(method.function(&this.ast))
            });
            this.top_level_scope = was_top_level;
          }
          ClassMember::ClassProp(prop) => {
            if prop.key(&this.ast).is_computed() {
              // webpack use `walk_expression`, `walk_expression` just walk down the ast, so it's ok to use `walk_prop_name`
              this.walk_prop_name(prop.key(&this.ast));
            }
            if let Some(value) = prop.value(&this.ast)
              && !this
                .plugin_drive
                .clone()
                .class_body_value(
                  this,
                  class_element,
                  value.span(&this.ast),
                  class_decl_or_expr,
                )
                .unwrap_or_default()
            {
              let was_top_level = this.top_level_scope;
              this.top_level_scope = TopLevelScope::False;
              this.walk_expression(value);
              this.top_level_scope = was_top_level;
            }
          }
          ClassMember::PrivateProp(prop) => {
            // prop.key is always not computed in private prop, so we don't need to walk it
            if let Some(value) = prop.value(&this.ast)
              && !this
                .plugin_drive
                .clone()
                .class_body_value(
                  this,
                  class_element,
                  value.span(&this.ast),
                  class_decl_or_expr,
                )
                .unwrap_or_default()
            {
              let was_top_level = this.top_level_scope;
              this.top_level_scope = TopLevelScope::False;
              this.walk_expression(value);
              this.top_level_scope = was_top_level;
            }
          }
          ClassMember::StaticBlock(block) => {
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            this.walk_block_statement(block.body(&this.ast));
            this.top_level_scope = was_top_level;
          }
          ClassMember::Empty(_) => {}
          ClassMember::AutoAccessor(_) => {}
        };
      }
    });
  }
}

fn member_prop_len(ast: &Ast, member_prop: MemberProp) -> Option<usize> {
  match member_prop {
    MemberProp::Ident(ident) => Some(ast.get_utf8(ident.sym(ast)).len()),
    MemberProp::PrivateName(name) => Some(ast.get_utf8(name.name(ast)).len() + 1),
    MemberProp::Computed(_) => None,
  }
}
