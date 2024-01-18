use std::borrow::Cow;

use swc_core::ecma::ast::{
  ArrayLit, ArrayPat, ArrowExpr, AssignExpr, AssignPat, AwaitExpr, BinExpr, BlockStmt,
  BlockStmtOrExpr, CallExpr, Callee, CatchClause, Class, ClassDecl, ClassExpr, ClassMember,
  CondExpr, Decl, DefaultDecl, DoWhileStmt, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr,
  ExprOrSpread, ExprStmt, FnDecl, FnExpr, ForHead, Function, Ident, KeyValuePatProp, KeyValueProp,
  MemberExpr, MemberProp, MetaPropExpr, NamedExport, NewExpr, ObjectLit, OptCall, OptChainBase,
  OptChainExpr, ParamOrTsParamProp, Pat, PatOrExpr, ThisExpr, UnaryOp,
};
use swc_core::ecma::ast::{ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt, WithStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, ObjectPat, ObjectPatProp, Stmt, WhileStmt};
use swc_core::ecma::ast::{Prop, PropName, PropOrSpread, RestPat, ReturnStmt, SeqExpr, TaggedTpl};
use swc_core::ecma::ast::{SwitchCase, SwitchStmt, TryStmt, VarDecl, VarDeclKind};
use swc_core::ecma::ast::{ThrowStmt, Tpl, UnaryExpr, UpdateExpr, VarDeclOrExpr, YieldExpr};

use super::JavascriptParser;
use crate::parser_plugin::{is_logic_op, JavaScriptParserPluginDrive, JavascriptParserPlugin};

fn warp_ident_to_pat(ident: Ident) -> Pat {
  Pat::Ident(ident.into())
}

impl<'ast, 'parser> JavascriptParser<'parser> {
  fn in_block_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;

    self.definitions = self.definitions_db.create_child(&old_definitions);
    f(self);

    self.definitions = old_definitions;
  }

  fn in_class_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = &'ast Pat>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;

    self.in_try = false;
    self.definitions = self.definitions_db.create_child(&old_definitions);

    if has_this {
      self.undefined_variable("this");
    }

    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.as_str());
    });

    f(self);

    self.in_try = old_in_try;
    self.definitions = old_definitions;
  }

  fn in_function_scope<I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = &'ast Pat>,
  {
    let old_definitions = self.definitions;
    self.definitions = self.definitions_db.create_child(&old_definitions);
    if has_this {
      self.undefined_variable("this");
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.as_str());
    });
    f(self);

    self.definitions = old_definitions;
  }

  pub fn walk_module_declarations(
    &mut self,
    statements: &'ast Vec<ModuleItem>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.walk_module_declaration(statement, plugin_drive);
    }
  }

  fn walk_module_declaration(
    &mut self,
    statement: &'ast ModuleItem,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match statement {
      ModuleItem::ModuleDecl(m) => {
        // TODO: `self.hooks.statement.call`
        match m {
          ModuleDecl::ExportDefaultDecl(decl) => {
            self.walk_export_default_declaration(decl, plugin_drive);
          }
          ModuleDecl::ExportDecl(decl) => self.walk_export_decl(decl, plugin_drive),
          ModuleDecl::ExportNamed(named) => self.walk_export_named_declaration(named, plugin_drive),
          ModuleDecl::ExportDefaultExpr(expr) => self.walk_export_default_expr(expr, plugin_drive),
          ModuleDecl::ExportAll(_) | ModuleDecl::Import(_) => (),
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }
      ModuleItem::Stmt(s) => self.walk_statement(s, plugin_drive),
    }
  }

  fn walk_export_decl(
    &mut self,
    expr: &'ast ExportDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match &expr.decl {
      Decl::Class(c) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_class(Some(&c.ident), &c.class, plugin_drive)
      }
      Decl::Fn(f) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_function_declaration(f, plugin_drive);
      }
      Decl::Var(decl) => self.walk_variable_declaration(decl, plugin_drive),
      Decl::Using(_) => (),
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!()
      }
    }
  }

  fn walk_export_default_expr(
    &mut self,
    expr: &'ast ExportDefaultExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `self.hooks.export.call`
    self.walk_expression(&expr.expr, plugin_drive);
  }

  fn walk_export_named_declaration(
    &mut self,
    _decl: &'ast NamedExport,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // self.walk_statement(decl);
  }

  fn walk_export_default_declaration(
    &mut self,
    decl: &'ast ExportDefaultDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `hooks.export.call`
    match &decl.decl {
      DefaultDecl::Class(c) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_class(c.ident.as_ref(), &c.class, plugin_drive)
      }
      DefaultDecl::Fn(f) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_function_expression(f, plugin_drive)
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }

    // TODO: `hooks.export_expression.call`
  }

  pub fn walk_statements(
    &mut self,
    statements: &'ast Vec<Stmt>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.walk_statement(statement, plugin_drive);
    }
  }

  fn walk_statement(
    &mut self,
    statement: &'ast Stmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `self.hooks.statement.call`

    match statement {
      Stmt::Block(stmt) => self.walk_block_statement(stmt, plugin_drive),
      Stmt::Decl(stmt) => match stmt {
        Decl::Class(decl) => self.walk_class_declaration(decl, plugin_drive),
        Decl::Fn(decl) => self.walk_function_declaration(decl, plugin_drive),
        Decl::Var(decl) => self.walk_variable_declaration(decl, plugin_drive),
        Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::DoWhile(stmt) => self.walk_do_while_statement(stmt, plugin_drive),
      Stmt::Expr(stmt) => self.walk_expression_statement(stmt, plugin_drive),
      Stmt::ForIn(stmt) => self.walk_for_in_statement(stmt, plugin_drive),
      Stmt::ForOf(stmt) => self.walk_for_of_statement(stmt, plugin_drive),
      Stmt::For(stmt) => self.walk_for_statement(stmt, plugin_drive),
      Stmt::If(stmt) => self.walk_if_statement(stmt, plugin_drive),
      Stmt::Labeled(stmt) => self.walk_labeled_statement(stmt, plugin_drive),
      Stmt::Return(stmt) => self.walk_return_statement(stmt, plugin_drive),
      Stmt::Switch(stmt) => self.walk_switch_statement(stmt, plugin_drive),
      Stmt::Throw(stmt) => self.walk_throw_stmt(stmt, plugin_drive),
      Stmt::Try(stmt) => self.walk_try_statement(stmt, plugin_drive),
      Stmt::While(stmt) => self.walk_while_statement(stmt, plugin_drive),
      Stmt::With(stmt) => self.walk_with_statement(stmt, plugin_drive),
      _ => (),
    }
  }

  fn walk_with_statement(
    &mut self,
    stmt: &'ast WithStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&stmt.obj, plugin_drive);
    self.walk_nested_statement(&stmt.body, plugin_drive);
  }

  fn walk_while_statement(
    &mut self,
    stmt: &'ast WhileStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&stmt.test, plugin_drive);
    self.walk_nested_statement(&stmt.body, plugin_drive);
  }

  fn walk_try_statement(
    &mut self,
    stmt: &'ast TryStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if self.in_try {
      // FIXME: webpack use `self.walk_statement(stmt.block, plugin_drive)`
      self.walk_block_statement(&stmt.block, plugin_drive);
    } else {
      self.in_try = true;
      // FIXME: webpack use `self.walk_statement(stmt.block, plugin_drive)`
      self.walk_block_statement(&stmt.block, plugin_drive);
      self.in_try = false;
    }

    if let Some(handler) = &stmt.handler {
      self.walk_catch_clause(handler, plugin_drive);
    }

    if let Some(finalizer) = &stmt.finalizer {
      // FIXME: webpack use `self.walk_statement(finalizer, plugin_drive)`
      self.walk_block_statement(finalizer, plugin_drive);
    }
  }

  fn walk_catch_clause(
    &mut self,
    catch_clause: &'ast CatchClause,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      if let Some(param) = &catch_clause.param {
        this.enter_pattern(param, |this, ident| {
          this.define_variable(ident.sym.as_str());
        });
        this.walk_pattern(param, plugin_drive)
      }
      this.block_pre_walk_statements(&catch_clause.body.stmts, plugin_drive);
      // FIXME: webpack use `this.walk_statement(catch_clause.body)`
      this.walk_block_statement(&catch_clause.body, plugin_drive);
    })
  }

  fn walk_switch_statement(
    &mut self,
    stmt: &'ast SwitchStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&stmt.discriminant, plugin_drive);
    self.walk_switch_cases(&stmt.cases, plugin_drive);
  }

  fn walk_switch_cases(
    &mut self,
    cases: &'ast Vec<SwitchCase>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      for case in cases {
        if !case.cons.is_empty() {
          this.block_pre_walk_statements(&case.cons, plugin_drive);
        }
      }
      for case in cases {
        if let Some(test) = &case.test {
          this.walk_expression(test, plugin_drive);
        }
        this.walk_statements(&case.cons, plugin_drive);
      }
    })
  }

  fn walk_return_statement(
    &mut self,
    stmt: &'ast ReturnStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(arg) = &stmt.arg {
      self.walk_expression(arg, plugin_drive);
    }
  }

  fn walk_throw_stmt(
    &mut self,
    stmt: &'ast ThrowStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&stmt.arg, plugin_drive);
  }

  fn walk_labeled_statement(
    &mut self,
    stmt: &'ast LabeledStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: self.hooks.label.get
    self.walk_nested_statement(&stmt.body, plugin_drive);
  }

  fn walk_if_statement(
    &mut self,
    stmt: &'ast IfStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    let old = self.in_if;
    self.in_if = true;
    if let Some(result) = plugin_drive.statement_if(self, stmt) {
      if result {
        self.walk_nested_statement(&stmt.cons, plugin_drive);
      } else if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt, plugin_drive);
      }
    } else {
      self.walk_expression(&stmt.test, plugin_drive);
      self.walk_nested_statement(&stmt.cons, plugin_drive);
      if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt, plugin_drive);
      }
    }
    self.in_if = old;
  }

  fn walk_for_statement(
    &mut self,
    stmt: &'ast ForStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      if let Some(init) = &stmt.init {
        match init {
          VarDeclOrExpr::VarDecl(decl) => {
            this.block_pre_walk_variable_declaration(decl, plugin_drive);
            this.walk_variable_declaration(decl, plugin_drive);
          }
          VarDeclOrExpr::Expr(expr) => this.walk_expression(expr, plugin_drive),
        }
      }
      if let Some(test) = &stmt.test {
        this.walk_expression(test, plugin_drive);
      }
      if let Some(update) = &stmt.update {
        this.walk_expression(update, plugin_drive);
      }
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts, plugin_drive);
        this.walk_statements(&body.stmts, plugin_drive);
      } else {
        this.walk_nested_statement(&stmt.body, plugin_drive);
      }
    });
  }

  fn walk_for_of_statement(
    &mut self,
    stmt: &'ast ForOfStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      this.walk_for_head(&stmt.left, plugin_drive);
      this.walk_expression(&stmt.right, plugin_drive);
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts, plugin_drive);
        this.walk_statements(&body.stmts, plugin_drive);
      } else {
        this.walk_nested_statement(&stmt.body, plugin_drive);
      }
    });
  }

  fn walk_for_in_statement(
    &mut self,
    stmt: &'ast ForInStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      this.walk_for_head(&stmt.left, plugin_drive);
      this.walk_expression(&stmt.right, plugin_drive);
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts, plugin_drive);
        this.walk_statements(&body.stmts, plugin_drive);
      } else {
        this.walk_nested_statement(&stmt.body, plugin_drive);
      }
    });
  }

  fn walk_for_head(
    &mut self,
    for_head: &'ast ForHead,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match &for_head {
      ForHead::VarDecl(decl) => {
        self.block_pre_walk_variable_declaration(decl, plugin_drive);
        self.walk_variable_declaration(decl, plugin_drive);
      }
      ForHead::Pat(pat) => self.walk_pattern(pat, plugin_drive),
      ForHead::UsingDecl(_) => (),
    }
  }

  fn walk_variable_declaration(
    &mut self,
    decl: &'ast VarDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for declarator in &decl.decls {
      // if let Some(renamed_identifier) = declarator
      //   .init
      //   .as_ref()
      //   .and_then(|init| self.get_rename_identifier(&init))
      //   && let Some(name) = declarator.name.as_ident()
      // {
      //   // TODO: can_rename hook
      //   // TODO: rename hook

      //   // if declarator.is_synthesized()
      // }
      if !plugin_drive
        .declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.walk_pattern(&declarator.name, plugin_drive);
        if let Some(init) = &declarator.init {
          self.walk_expression(init, plugin_drive);
        }
      }
    }
  }

  fn walk_expression_statement(
    &mut self,
    stmt: &'ast ExprStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&stmt.expr, plugin_drive);
  }

  pub fn walk_expression(
    &mut self,
    expr: &'ast Expr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match expr {
      Expr::Array(expr) => self.walk_array_expression(expr, plugin_drive),
      Expr::Arrow(expr) => self.walk_arrow_function_expression(expr, plugin_drive),
      Expr::Assign(expr) => self.walk_assignment_expression(expr, plugin_drive),
      Expr::Await(expr) => self.walk_await_expression(expr, plugin_drive),
      Expr::Bin(expr) => self.walk_binary_expression(expr, plugin_drive),
      Expr::Call(expr) => self.walk_call_expression(expr, plugin_drive),
      Expr::Class(expr) => self.walk_class_expression(expr, plugin_drive),
      Expr::Cond(expr) => self.walk_conditional_expression(expr, plugin_drive),
      Expr::Fn(expr) => self.walk_function_expression(expr, plugin_drive),
      Expr::Ident(expr) => self.walk_identifier(expr, plugin_drive),
      Expr::MetaProp(expr) => self.walk_meta_property(expr, plugin_drive),
      Expr::Member(expr) => self.walk_member_expression(expr, plugin_drive),
      Expr::New(expr) => self.walk_new_expression(expr, plugin_drive),
      Expr::Object(expr) => self.walk_object_expression(expr, plugin_drive),
      Expr::OptChain(expr) => self.walk_chain_expression(expr, plugin_drive),
      Expr::Seq(expr) => self.walk_sequence_expression(expr, plugin_drive),
      Expr::TaggedTpl(expr) => self.walk_tagged_template_expression(expr, plugin_drive),
      Expr::Tpl(expr) => self.walk_template_expression(expr, plugin_drive),
      Expr::This(expr) => self.walk_this_expression(expr, plugin_drive),
      Expr::Unary(expr) => self.walk_unary_expression(expr, plugin_drive),
      Expr::Update(expr) => self.walk_update_expression(expr, plugin_drive),
      Expr::Yield(expr) => self.walk_yield_expression(expr, plugin_drive),
      Expr::Paren(expr) => self.walk_expression(&expr.expr, plugin_drive),
      Expr::SuperProp(_) | Expr::Lit(_) | Expr::PrivateName(_) | Expr::Invalid(_) => (),
      Expr::JSXMember(_)
      | Expr::JSXNamespacedName(_)
      | Expr::JSXEmpty(_)
      | Expr::JSXElement(_)
      | Expr::JSXFragment(_)
      | Expr::TsTypeAssertion(_)
      | Expr::TsConstAssertion(_)
      | Expr::TsNonNull(_)
      | Expr::TsAs(_)
      | Expr::TsInstantiation(_)
      | Expr::TsSatisfies(_) => unreachable!(),
    }
  }

  fn walk_yield_expression(
    &mut self,
    expr: &'ast YieldExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(arg) = &expr.arg {
      self.walk_expression(arg, plugin_drive);
    }
  }

  fn walk_update_expression(
    &mut self,
    expr: &'ast UpdateExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&expr.arg, plugin_drive);
  }

  fn walk_unary_expression(
    &mut self,
    expr: &'ast UnaryExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if expr.op == UnaryOp::TypeOf {
      // TODO: call_hooks_from_expression
      if plugin_drive.r#typeof(self, expr).unwrap_or_default() {
        return;
      }
      // TODO: expr.arg belongs chain_expression
    }
    self.walk_expression(&expr.arg, plugin_drive);
  }

  fn walk_this_expression(
    &mut self,
    _expr: &'ast ThisExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `this.hooks.call_hooks_for_names`
  }

  fn walk_template_expression(
    &mut self,
    expr: &'ast Tpl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    let exprs = expr.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs, plugin_drive);
  }

  fn walk_tagged_template_expression(
    &mut self,
    expr: &'ast TaggedTpl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&expr.tag, plugin_drive);

    let exprs = expr.tpl.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs, plugin_drive);
  }

  fn walk_sequence_expression(
    &mut self,
    expr: &'ast SeqExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    let exprs = expr.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs, plugin_drive);
  }

  fn walk_object_expression(
    &mut self,
    expr: &'ast ObjectLit,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for prop in &expr.props {
      self.walk_property_or_spread(prop, plugin_drive);
    }
  }

  fn walk_property_or_spread(
    &mut self,
    prop: &'ast PropOrSpread,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match &prop {
      PropOrSpread::Spread(spread) => self.walk_expression(&spread.expr, plugin_drive),
      PropOrSpread::Prop(prop) => self.walk_property(prop, plugin_drive),
    }
  }

  fn walk_key_value_prop(
    &mut self,
    kv: &'ast KeyValueProp,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if kv.key.is_computed() {
      // FIXME: webpack use `walk_expression` here
      self.walk_prop_name(&kv.key, plugin_drive);
    }
    self.walk_expression(&kv.value, plugin_drive);
  }

  fn walk_property(
    &mut self,
    prop: &'ast Prop,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match prop {
      Prop::Shorthand(ident) => {
        self.in_short_hand = true;
        self.walk_identifier(ident, plugin_drive);
        self.in_short_hand = false;
      }
      Prop::KeyValue(kv) => self.walk_key_value_prop(kv, plugin_drive),
      Prop::Assign(assign) => self.walk_expression(&assign.value, plugin_drive),
      Prop::Getter(getter) => {
        if let Some(body) = &getter.body {
          self.walk_block_statement(body, plugin_drive);
        }
      }
      Prop::Setter(seeter) => {
        if let Some(body) = &seeter.body {
          self.walk_block_statement(body, plugin_drive);
        }
      }
      Prop::Method(method) => {
        self.walk_prop_name(&method.key, plugin_drive);
        // FIXME: maybe we need in_function_scope here
        self.walk_function(&method.function, plugin_drive);
      }
    }
  }

  fn walk_prop_name(
    &mut self,
    prop_name: &'ast PropName,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(computed) = prop_name.as_computed() {
      self.walk_expression(&computed.expr, plugin_drive);
    }
  }

  fn walk_new_expression(
    &mut self,
    expr: &'ast NewExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `callHooksForExpression`
    if plugin_drive.new_expression(self, expr).unwrap_or_default() {
      return;
    }
    self.walk_expression(&expr.callee, plugin_drive);
    if let Some(args) = &expr.args {
      self.walk_expr_or_spread(args, plugin_drive);
    }
  }

  fn walk_meta_property(
    &mut self,
    _expr: &'ast MetaPropExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: hooks call
  }

  fn walk_conditional_expression(
    &mut self,
    expr: &'ast CondExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: self.hooks.expression_conditional_operation.call

    self.walk_expression(&expr.test, plugin_drive);
    self.walk_expression(&expr.cons, plugin_drive);
    self.walk_expression(&expr.alt, plugin_drive);
  }

  fn walk_class_expression(
    &mut self,
    expr: &'ast ClassExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_class(expr.ident.as_ref(), &expr.class, plugin_drive);
  }

  fn walk_chain_expression(
    &mut self,
    expr: &'ast OptChainExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `let result = this.hooks.optional_chaining.call(expression)`
    match &*expr.base {
      OptChainBase::Call(call) => self.walk_opt_call(call, plugin_drive),
      OptChainBase::Member(member) => self.walk_member_expression(member, plugin_drive),
    };
  }

  fn walk_member_expression(
    &mut self,
    expr: &'ast MemberExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // FIXME: should remove:
    if plugin_drive.member(self, expr).unwrap_or_default() {
      return;
    }

    // TODO: member expression info
    self.walk_expression(&expr.obj, plugin_drive);
    if let MemberProp::Computed(computed) = &expr.prop {
      self.walk_expression(&computed.expr, plugin_drive);
    }
  }

  fn walk_opt_call(
    &mut self,
    call: &'ast OptCall,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: should align to walkCallExpression in webpack.
    self.walk_expression(&call.callee, plugin_drive);
    self.walk_expr_or_spread(&call.args, plugin_drive);
  }

  fn walk_call_expression(
    &mut self,
    expr: &'ast CallExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // FIXME: should align to webpack
    if let Some(result) = plugin_drive.call(self, expr) {
      if !result {
        self.walk_expr_or_spread(&expr.args, plugin_drive);
      }
      return;
    }

    match &expr.callee {
      Callee::Super(_) => (),
      Callee::Import(_import) => {
        // TODO: `if (this.hooks.importCall.call(expression)) { return }`
      }
      Callee::Expr(expr) => {
        // TODO: `hooks.callMemberChain`
        self.walk_expression(expr, plugin_drive);
      }
    }
    self.walk_expr_or_spread(&expr.args, plugin_drive);
  }

  fn walk_expr_or_spread(
    &mut self,
    args: &'ast Vec<ExprOrSpread>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for arg in args {
      self.walk_expression(&arg.expr, plugin_drive);
    }
  }

  fn walk_left_right_expression(
    &mut self,
    expr: &'ast BinExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&expr.left, plugin_drive);
    self.walk_expression(&expr.right, plugin_drive);
  }

  fn walk_binary_expression(
    &mut self,
    expr: &'ast BinExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if is_logic_op(expr.op) {
      if let Some(keep_right) = plugin_drive.expression_logical_operator(self, expr) {
        if keep_right {
          self.walk_expression(&expr.right, plugin_drive);
        }
      } else {
        self.walk_left_right_expression(expr, plugin_drive);
      }
    } else if plugin_drive.binary_expression(self, expr).is_none() {
      self.walk_left_right_expression(expr, plugin_drive);
    }
  }

  fn walk_await_expression(
    &mut self,
    expr: &'ast AwaitExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: if (this.scope.topLevelScope === true)
    // this.hooks.topLevelAwait.call(expression);
    self.walk_expression(&expr.arg, plugin_drive);
  }

  fn walk_identifier(
    &mut self,
    _identifier: &'ast Ident,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: self.call_hooks_for_name
  }

  // fn get_rename_identifier(&mut self, expr: &Expr) -> Option<String> {
  //   let result = self.evaluate_expression(expr);
  //   result
  //     .is_identifier()
  //     .then(|| result.identifier().to_string())
  // }

  fn walk_assignment_expression(
    &mut self,
    expr: &'ast AssignExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(ident) = expr.left.as_ident() {
      // if let Some(rename_identifier) = self.get_rename_identifier(&expr.right) {
      //     // TODO:
      //   }

      self.walk_expression(&expr.right, plugin_drive);
      self.enter_ident(ident, |this, ident| {
        // TODO: if (!this.callHooksForName(this.hooks.assign, name, expression)) {
        // FIXME: webpack use `self.walk_expression`
        this.walk_identifier(ident, plugin_drive);
      });
    } else if let Some(pat) = expr.left.as_pat() {
      self.walk_expression(&expr.right, plugin_drive);
      self.enter_pattern(pat, |this, ident| {
        // TODO: if (!this.callHooksForName(this.hooks.assign, name, expression)) {
        this.define_variable(ident.sym.as_str());
      })
    } else {
      self.walk_expression(&expr.right, plugin_drive);
      match &expr.left {
        PatOrExpr::Expr(expr) => self.walk_expression(expr, plugin_drive),
        PatOrExpr::Pat(pat) => self.walk_pattern(pat, plugin_drive),
      }
    }
    // TODO:
    // else if let Some(member) = expr.left.as_expr().and_then(|expr| expr.as_member()) {
    // }
  }

  fn walk_arrow_function_expression(
    &mut self,
    expr: &'ast ArrowExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_function_scope(false, expr.params.iter(), |this| {
      for param in &expr.params {
        this.walk_pattern(param, plugin_drive);
      }
      match &*expr.body {
        BlockStmtOrExpr::BlockStmt(stmt) => {
          this.detect_mode(&stmt.stmts);
          // FIXME: webpack use `pre_walk_statement` here
          this.pre_walk_block_statement(stmt, plugin_drive);
          // FIXME: webpack use `walk_statement` here
          this.walk_block_statement(stmt, plugin_drive);
        }
        BlockStmtOrExpr::Expr(expr) => this.walk_expression(expr, plugin_drive),
      }
    })
  }

  fn walk_expressions<I>(
    &mut self,
    expressions: I,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) where
    I: Iterator<Item = &'ast Expr>,
  {
    for expr in expressions {
      self.walk_expression(expr, plugin_drive)
    }
  }

  fn walk_array_expression(
    &mut self,
    expr: &'ast ArrayLit,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    expr
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.walk_expression(&ele.expr, plugin_drive))
  }

  fn walk_nested_statement(
    &mut self,
    stmt: &'ast Stmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: self.prev_statement = undefined;
    self.walk_statement(stmt, plugin_drive);
  }

  fn walk_do_while_statement(
    &mut self,
    stmt: &'ast DoWhileStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_nested_statement(&stmt.body, plugin_drive);
    self.walk_expression(&stmt.test, plugin_drive);
  }

  fn walk_block_statement(
    &mut self,
    stmt: &'ast BlockStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_block_scope(|this| {
      this.block_pre_walk_statements(&stmt.stmts, plugin_drive);
      this.walk_statements(&stmt.stmts, plugin_drive);
    })
  }

  fn walk_function_declaration(
    &mut self,
    decl: &'ast FnDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.in_function_scope(
      true,
      decl.function.params.iter().map(|param| &param.pat),
      |this| {
        this.walk_function(&decl.function, plugin_drive);
      },
    )
  }

  fn walk_function(
    &mut self,
    f: &'ast Function,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for param in &f.params {
      self.walk_pattern(&param.pat, plugin_drive)
    }
    if let Some(body) = &f.body {
      self.detect_mode(&body.stmts);
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_block_statement(body, plugin_drive);
      // FIXME: webpack use `walk_statement` here
      self.walk_block_statement(body, plugin_drive);
    }
  }

  fn walk_function_expression(
    &mut self,
    expr: &'ast FnExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    let mut scope_params: Vec<_> = expr
      .function
      .params
      .iter()
      .map(|params| Cow::Borrowed(&params.pat))
      .collect();

    if let Some(pat) = expr
      .ident
      .as_ref()
      .map(|ident| warp_ident_to_pat(ident.clone()))
    {
      scope_params.push(Cow::Owned(pat));
    }

    self.in_function_scope(
      true,
      scope_params.iter().map(|param| param.as_ref()),
      |this| {
        this.walk_function(&expr.function, plugin_drive);
      },
    );
  }

  fn walk_pattern(
    &mut self,
    pat: &'ast Pat,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match pat {
      Pat::Array(array) => self.walk_array_pattern(array, plugin_drive),
      Pat::Assign(assign) => self.walk_assignment_pattern(assign, plugin_drive),
      Pat::Object(obj) => self.walk_object_pattern(obj, plugin_drive),
      Pat::Rest(rest) => self.walk_rest_element(rest, plugin_drive),
      _ => (),
    }
  }

  fn walk_rest_element(
    &mut self,
    rest: &'ast RestPat,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_pattern(&rest.arg, plugin_drive);
  }

  fn walk_object_pattern(
    &mut self,
    obj: &'ast ObjectPat,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for prop in &obj.props {
      match prop {
        ObjectPatProp::KeyValue(kv) => {
          if kv.key.is_computed() {
            // FIXME: webpack use `walk_expression` here
            self.walk_prop_name(&kv.key, plugin_drive);
          }
          self.walk_pattern(&kv.value, plugin_drive);
        }
        ObjectPatProp::Assign(assign) => {
          if let Some(value) = &assign.value {
            self.walk_expression(value, plugin_drive);
          }
        }
        ObjectPatProp::Rest(rest) => self.walk_rest_element(rest, plugin_drive),
      }
    }
  }

  fn walk_assignment_pattern(
    &mut self,
    pat: &'ast AssignPat,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_expression(&pat.right, plugin_drive);
    self.walk_pattern(&pat.left, plugin_drive);
  }

  fn walk_array_pattern(
    &mut self,
    pat: &'ast ArrayPat,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    pat
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.walk_pattern(ele, plugin_drive));
  }

  fn walk_class_declaration(
    &mut self,
    decl: &'ast ClassDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.walk_class(Some(&decl.ident), &decl.class, plugin_drive);
  }

  fn walk_class(
    &mut self,
    ident: Option<&'ast Ident>,
    classy: &'ast Class,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(super_class) = &classy.super_class {
      // TODO: `hooks.class_extends_expression`
      self.walk_expression(super_class, plugin_drive);
    }

    let scope_params = if let Some(pat) = ident.map(|ident| warp_ident_to_pat(ident.clone())) {
      vec![pat]
    } else {
      vec![]
    };

    self.in_class_scope(true, scope_params.iter(), |this| {
      for class_element in &classy.body {
        // TODO: `hooks.class_body_element`
        match class_element {
          ClassMember::Constructor(ctor) => {
            if ctor.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&ctor.key, plugin_drive);
            }
            for prop in &ctor.params {
              match prop {
                ParamOrTsParamProp::Param(param) => this.walk_pattern(&param.pat, plugin_drive),
                ParamOrTsParamProp::TsParamProp(_) => unreachable!(),
              }
            }
            // TODO: `hooks.body_value`;
            if let Some(body) = &ctor.body {
              this.walk_block_statement(body, plugin_drive);
            }
          }
          ClassMember::Method(method) => {
            if method.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&method.key, plugin_drive);
            }
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| &p.pat),
              |this| {
                // TODO: `hooks.body_value`;
                if let Some(body) = &method.function.body {
                  this.walk_block_statement(body, plugin_drive);
                }
              },
            );
          }
          ClassMember::PrivateMethod(method) => {
            this.walk_identifier(&method.key.id, plugin_drive);
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| &p.pat),
              |this| {
                // TODO: `hooks.body_value`;
                if let Some(body) = &method.function.body {
                  this.walk_block_statement(body, plugin_drive);
                }
              },
            );
          }
          ClassMember::ClassProp(prop) => {
            if prop.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&prop.key, plugin_drive);
            }
            if let Some(value) = &prop.value {
              this.walk_expression(value, plugin_drive);
            }
          }
          ClassMember::PrivateProp(prop) => {
            this.walk_identifier(&prop.key.id, plugin_drive);
            if let Some(value) = &prop.value {
              this.walk_expression(value, plugin_drive);
            }
          }
          ClassMember::Empty(_) => {}
          ClassMember::AutoAccessor(_) => {}
          ClassMember::StaticBlock(block) => this.walk_block_statement(&block.body, plugin_drive),
          ClassMember::TsIndexSignature(_) => unreachable!(),
        };
      }
    });
  }
}
