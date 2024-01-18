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
use crate::parser_plugin::{is_logic_op, JavascriptParserPlugin};

fn warp_ident_to_pat(ident: Ident) -> Pat {
  Pat::Ident(ident.into())
}

impl<'parser> JavascriptParser<'parser> {
  fn in_block_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;

    self.definitions = self.definitions_db.create_child(&old_definitions);
    f(self);

    self.definitions = old_definitions;
  }

  fn in_class_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;

    self.in_try = false;
    self.definitions = self.definitions_db.create_child(&old_definitions);

    if has_this {
      self.undefined_variable("this".to_string());
    }

    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.to_string());
    });

    f(self);

    self.in_try = old_in_try;
    self.definitions = old_definitions;
  }

  fn in_function_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;

    self.definitions = self.definitions_db.create_child(&old_definitions);
    if has_this {
      self.undefined_variable("this".to_string());
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.to_string());
    });
    f(self);

    self.definitions = old_definitions;
  }

  pub fn walk_module_declarations(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.walk_module_declaration(statement);
    }
  }

  fn walk_module_declaration(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(m) => {
        // TODO: `self.hooks.statement.call`
        match m {
          ModuleDecl::ExportDefaultDecl(decl) => {
            self.walk_export_default_declaration(decl);
          }
          ModuleDecl::ExportDecl(decl) => self.walk_export_decl(decl),
          ModuleDecl::ExportNamed(named) => self.walk_export_named_declaration(named),
          ModuleDecl::ExportDefaultExpr(expr) => self.walk_export_default_expr(expr),
          ModuleDecl::ExportAll(_) | ModuleDecl::Import(_) => (),
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }
      ModuleItem::Stmt(s) => self.walk_statement(s),
    }
  }

  fn walk_export_decl(&mut self, expr: &ExportDecl) {
    match &expr.decl {
      Decl::Class(c) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_class(Some(&c.ident), &c.class)
      }
      Decl::Fn(f) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_function_declaration(f);
      }
      Decl::Var(decl) => self.walk_variable_declaration(decl),
      Decl::Using(_) => (),
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!()
      }
    }
  }

  fn walk_export_default_expr(&mut self, expr: &ExportDefaultExpr) {
    // TODO: `self.hooks.export.call`
    self.walk_expression(&expr.expr);
  }

  fn walk_export_named_declaration(&mut self, _decl: &NamedExport) {
    // self.walk_statement(decl)
  }

  fn walk_export_default_declaration(&mut self, decl: &ExportDefaultDecl) {
    // TODO: `hooks.export.call`
    match &decl.decl {
      DefaultDecl::Class(c) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_class(c.ident.as_ref(), &c.class)
      }
      DefaultDecl::Fn(f) => {
        // FIXME: webpack use `self.walk_statement` here
        self.walk_function_expression(f)
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }

    // TODO: `hooks.export_expression.call`
  }

  pub fn walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.walk_statement(statement);
    }
  }

  fn walk_statement(&mut self, statement: &Stmt) {
    // TODO: `self.hooks.statement.call`

    match statement {
      Stmt::Block(stmt) => self.walk_block_statement(stmt),
      Stmt::Decl(stmt) => match stmt {
        Decl::Class(decl) => self.walk_class_declaration(decl),
        Decl::Fn(decl) => self.walk_function_declaration(decl),
        Decl::Var(decl) => self.walk_variable_declaration(decl),
        Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::DoWhile(stmt) => self.walk_do_while_statement(stmt),
      Stmt::Expr(stmt) => self.walk_expression_statement(stmt),
      Stmt::ForIn(stmt) => self.walk_for_in_statement(stmt),
      Stmt::ForOf(stmt) => self.walk_for_of_statement(stmt),
      Stmt::For(stmt) => self.walk_for_statement(stmt),
      Stmt::If(stmt) => self.walk_if_statement(stmt),
      Stmt::Labeled(stmt) => self.walk_labeled_statement(stmt),
      Stmt::Return(stmt) => self.walk_return_statement(stmt),
      Stmt::Switch(stmt) => self.walk_switch_statement(stmt),
      Stmt::Throw(stmt) => self.walk_throw_stmt(stmt),
      Stmt::Try(stmt) => self.walk_try_statement(stmt),
      Stmt::While(stmt) => self.walk_while_statement(stmt),
      Stmt::With(stmt) => self.walk_with_statement(stmt),
      _ => (),
    }
  }

  fn walk_with_statement(&mut self, stmt: &WithStmt) {
    self.walk_expression(&stmt.obj);
    self.walk_nested_statement(&stmt.body);
  }

  fn walk_while_statement(&mut self, stmt: &WhileStmt) {
    self.walk_expression(&stmt.test);
    self.walk_nested_statement(&stmt.body);
  }

  fn walk_try_statement(&mut self, stmt: &TryStmt) {
    if self.in_try {
      // FIXME: webpack use `self.walk_statement(stmt.block)`
      self.walk_block_statement(&stmt.block);
    } else {
      self.in_try = true;
      // FIXME: webpack use `self.walk_statement(stmt.block)`
      self.walk_block_statement(&stmt.block);
      self.in_try = false;
    }

    if let Some(handler) = &stmt.handler {
      self.walk_catch_clause(handler);
    }

    if let Some(finalizer) = &stmt.finalizer {
      // FIXME: webpack use `self.walk_statement(finalizer)`
      self.walk_block_statement(finalizer);
    }
  }

  fn walk_catch_clause(&mut self, catch_clause: &CatchClause) {
    self.in_block_scope(|this| {
      if let Some(param) = &catch_clause.param {
        this.enter_pattern(Cow::Borrowed(param), |this, ident| {
          this.define_variable(ident.sym.to_string());
        });
        this.walk_pattern(param)
      }
      this.block_pre_walk_statements(&catch_clause.body.stmts);
      // FIXME: webpack use `this.walk_statement(catch_clause.body)`
      this.walk_block_statement(&catch_clause.body);
    })
  }

  fn walk_switch_statement(&mut self, stmt: &SwitchStmt) {
    self.walk_expression(&stmt.discriminant);
    self.walk_switch_cases(&stmt.cases);
  }

  fn walk_switch_cases(&mut self, cases: &Vec<SwitchCase>) {
    self.in_block_scope(|this| {
      for case in cases {
        if !case.cons.is_empty() {
          this.block_pre_walk_statements(&case.cons);
        }
      }
      for case in cases {
        if let Some(test) = &case.test {
          this.walk_expression(test);
        }
        this.walk_statements(&case.cons);
      }
    })
  }

  fn walk_return_statement(&mut self, stmt: &ReturnStmt) {
    if let Some(arg) = &stmt.arg {
      self.walk_expression(arg);
    }
  }

  fn walk_throw_stmt(&mut self, stmt: &ThrowStmt) {
    self.walk_expression(&stmt.arg);
  }

  fn walk_labeled_statement(&mut self, stmt: &LabeledStmt) {
    // TODO: self.hooks.label.get
    self.walk_nested_statement(&stmt.body);
  }

  fn walk_if_statement(&mut self, stmt: &IfStmt) {
    let old = self.in_if;
    self.in_if = true;
    if let Some(result) = self.plugin_drive.clone().statement_if(self, stmt) {
      if result {
        self.walk_nested_statement(&stmt.cons);
      } else if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt);
      }
    } else {
      self.walk_expression(&stmt.test);
      self.walk_nested_statement(&stmt.cons);
      if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt);
      }
    }
    self.in_if = old;
  }

  fn walk_for_statement(&mut self, stmt: &ForStmt) {
    self.in_block_scope(|this| {
      if let Some(init) = &stmt.init {
        match init {
          VarDeclOrExpr::VarDecl(decl) => {
            this.block_pre_walk_variable_declaration(decl);
            this.walk_variable_declaration(decl);
          }
          VarDeclOrExpr::Expr(expr) => this.walk_expression(expr),
        }
      }
      if let Some(test) = &stmt.test {
        this.walk_expression(test)
      }
      if let Some(update) = &stmt.update {
        this.walk_expression(update)
      }
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts);
        this.walk_statements(&body.stmts);
      } else {
        this.walk_nested_statement(&stmt.body);
      }
    });
  }

  fn walk_for_of_statement(&mut self, stmt: &ForOfStmt) {
    self.in_block_scope(|this| {
      this.walk_for_head(&stmt.left);
      this.walk_expression(&stmt.right);
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts);
        this.walk_statements(&body.stmts);
      } else {
        this.walk_nested_statement(&stmt.body);
      }
    });
  }

  fn walk_for_in_statement(&mut self, stmt: &ForInStmt) {
    self.in_block_scope(|this| {
      this.walk_for_head(&stmt.left);
      this.walk_expression(&stmt.right);
      if let Some(body) = stmt.body.as_block() {
        this.block_pre_walk_statements(&body.stmts);
        this.walk_statements(&body.stmts);
      } else {
        this.walk_nested_statement(&stmt.body);
      }
    });
  }

  fn walk_for_head(&mut self, for_head: &ForHead) {
    match &for_head {
      ForHead::VarDecl(decl) => {
        self.block_pre_walk_variable_declaration(decl);
        self.walk_variable_declaration(decl);
      }
      ForHead::Pat(pat) => self.walk_pattern(pat),
      ForHead::UsingDecl(_) => (),
    }
  }

  fn walk_variable_declaration(&mut self, decl: &VarDecl) {
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
      if !self
        .plugin_drive
        .clone()
        .declarator(self, declarator, decl)
        .unwrap_or_default()
      {
        self.walk_pattern(&declarator.name);
        if let Some(init) = &declarator.init {
          self.walk_expression(init);
        }
      }
    }
  }

  fn walk_expression_statement(&mut self, stmt: &ExprStmt) {
    self.walk_expression(&stmt.expr);
  }

  pub fn walk_expression(&mut self, expr: &Expr) {
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
      Expr::Paren(expr) => self.walk_expression(&expr.expr),
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

  fn walk_yield_expression(&mut self, expr: &YieldExpr) {
    if let Some(arg) = &expr.arg {
      self.walk_expression(arg);
    }
  }

  fn walk_update_expression(&mut self, expr: &UpdateExpr) {
    self.walk_expression(&expr.arg)
  }

  fn walk_unary_expression(&mut self, expr: &UnaryExpr) {
    if expr.op == UnaryOp::TypeOf {
      // TODO: call_hooks_from_expression
      if self
        .plugin_drive
        .clone()
        .r#typeof(self, expr)
        .unwrap_or_default()
      {
        return;
      }
      // TODO: expr.arg belongs chain_expression
    }
    self.walk_expression(&expr.arg)
  }

  fn walk_this_expression(&mut self, _expr: &ThisExpr) {
    // TODO: `this.hooks.call_hooks_for_names`
  }

  fn walk_template_expression(&mut self, expr: &Tpl) {
    let exprs = expr.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs);
  }

  fn walk_tagged_template_expression(&mut self, expr: &TaggedTpl) {
    self.walk_expression(&expr.tag);

    let exprs = expr.tpl.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs);
  }

  fn walk_sequence_expression(&mut self, expr: &SeqExpr) {
    let exprs = expr.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs);
  }

  fn walk_object_expression(&mut self, expr: &ObjectLit) {
    for prop in &expr.props {
      self.walk_property_or_spread(prop);
    }
  }

  fn walk_property_or_spread(&mut self, prop: &PropOrSpread) {
    match &prop {
      PropOrSpread::Spread(spread) => self.walk_expression(&spread.expr),
      PropOrSpread::Prop(prop) => self.walk_property(prop),
    }
  }

  fn walk_key_value_prop(&mut self, kv: &KeyValueProp) {
    if kv.key.is_computed() {
      // FIXME: webpack use `walk_expression` here
      self.walk_prop_name(&kv.key);
    }
    self.walk_expression(&kv.value);
  }

  fn walk_property(&mut self, prop: &Prop) {
    match prop {
      Prop::Shorthand(ident) => {
        self.in_short_hand = true;
        self.walk_identifier(ident);
        self.in_short_hand = false;
      }
      Prop::KeyValue(kv) => self.walk_key_value_prop(kv),
      Prop::Assign(assign) => self.walk_expression(&assign.value),
      Prop::Getter(getter) => {
        if let Some(body) = &getter.body {
          self.walk_block_statement(body);
        }
      }
      Prop::Setter(seeter) => {
        if let Some(body) = &seeter.body {
          self.walk_block_statement(body);
        }
      }
      Prop::Method(method) => {
        self.walk_prop_name(&method.key);
        // FIXME: maybe we need in_function_scope here
        self.walk_function(&method.function);
      }
    }
  }

  fn walk_prop_name(&mut self, prop_name: &PropName) {
    if let Some(computed) = prop_name.as_computed() {
      self.walk_expression(&computed.expr);
    }
  }

  fn walk_new_expression(&mut self, expr: &NewExpr) {
    // TODO: `callHooksForExpression`
    if self
      .plugin_drive
      .clone()
      .new_expression(self, expr)
      .unwrap_or_default()
    {
      return;
    }
    self.walk_expression(&expr.callee);
    if let Some(args) = &expr.args {
      self.walk_expr_or_spread(args);
    }
  }

  fn walk_meta_property(&mut self, _expr: &MetaPropExpr) {
    // TODO: hooks call
  }

  fn walk_conditional_expression(&mut self, expr: &CondExpr) {
    // TODO: self.hooks.expression_conditional_operation.call

    self.walk_expression(&expr.test);
    self.walk_expression(&expr.cons);
    self.walk_expression(&expr.alt);
  }

  fn walk_class_expression(&mut self, expr: &ClassExpr) {
    self.walk_class(expr.ident.as_ref(), &expr.class);
  }

  fn walk_chain_expression(&mut self, expr: &OptChainExpr) {
    // TODO: `let result = this.hooks.optional_chaining.call(expression)`
    match &*expr.base {
      OptChainBase::Call(call) => self.walk_opt_call(call),
      OptChainBase::Member(member) => self.walk_member_expression(member),
    };
  }

  fn walk_member_expression(&mut self, expr: &MemberExpr) {
    // FIXME: should remove:
    if self
      .plugin_drive
      .clone()
      .member(self, expr)
      .unwrap_or_default()
    {
      return;
    }

    // TODO: member expression info
    self.walk_expression(&expr.obj);
    if let MemberProp::Computed(computed) = &expr.prop {
      self.walk_expression(&computed.expr)
    }
  }

  fn walk_opt_call(&mut self, call: &OptCall) {
    // TODO: should align to walkCallExpression in webpack.
    self.walk_expression(&call.callee);
    self.walk_expr_or_spread(&call.args);
  }

  fn walk_call_expression(&mut self, expr: &CallExpr) {
    // FIXME: should align to webpack
    if let Some(result) = self.plugin_drive.clone().call(self, expr) {
      if !result {
        self.walk_expr_or_spread(&expr.args);
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
        self.walk_expression(expr)
      }
    }
    self.walk_expr_or_spread(&expr.args);
  }

  fn walk_expr_or_spread(&mut self, args: &Vec<ExprOrSpread>) {
    for arg in args {
      self.walk_expression(&arg.expr)
    }
  }

  fn walk_left_right_expression(&mut self, expr: &BinExpr) {
    self.walk_expression(&expr.left);
    self.walk_expression(&expr.right);
  }

  fn walk_binary_expression(&mut self, expr: &BinExpr) {
    if is_logic_op(expr.op) {
      if let Some(keep_right) = self
        .plugin_drive
        .clone()
        .expression_logical_operator(self, expr)
      {
        if keep_right {
          self.walk_expression(&expr.right);
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

  fn walk_await_expression(&mut self, expr: &AwaitExpr) {
    // TODO: if (this.scope.topLevelScope === true)
    // this.hooks.topLevelAwait.call(expression);
    self.walk_expression(&expr.arg);
  }

  fn walk_identifier(&mut self, identifier: &Ident) {
    // TODO: self.call_hooks_for_name
    self.plugin_drive.clone().identifier(self, identifier);
  }

  // fn get_rename_identifier(&mut self, expr: &Expr) -> Option<String> {
  //   let result = self.evaluate_expression(expr);
  //   result
  //     .is_identifier()
  //     .then(|| result.identifier().to_string())
  // }

  fn walk_assignment_expression(&mut self, expr: &AssignExpr) {
    if let Some(ident) = expr.left.as_ident() {
      // if let Some(rename_identifier) = self.get_rename_identifier(&expr.right) {
      //     // TODO:
      //   }

      self.walk_expression(&expr.right);
      self.enter_pattern(
        Cow::Owned(warp_ident_to_pat(ident.clone())),
        |this, ident| {
          // TODO: if (!this.callHooksForName(this.hooks.assign, name, expression)) {
          // FIXME: webpack use `self.walk_expression`
          this.walk_identifier(ident);
        },
      );
    } else if let Some(pat) = expr.left.as_pat() {
      self.walk_expression(&expr.right);
      self.enter_pattern(Cow::Borrowed(pat), |this, ident| {
        // TODO: if (!this.callHooksForName(this.hooks.assign, name, expression)) {
        this.define_variable(ident.sym.to_string());
      })
    } else {
      self.walk_expression(&expr.right);
      match &expr.left {
        PatOrExpr::Expr(expr) => self.walk_expression(expr),
        PatOrExpr::Pat(pat) => self.walk_pattern(pat),
      }
    }
    // TODO:
    // else if let Some(member) = expr.left.as_expr().and_then(|expr| expr.as_member()) {
    // }
  }

  fn walk_arrow_function_expression(&mut self, expr: &ArrowExpr) {
    self.in_function_scope(false, expr.params.iter().map(Cow::Borrowed), |this| {
      for param in &expr.params {
        this.walk_pattern(param)
      }
      match &*expr.body {
        BlockStmtOrExpr::BlockStmt(stmt) => {
          this.detect_mode(&stmt.stmts);
          // FIXME: webpack use `pre_walk_statement` here
          this.pre_walk_block_statement(stmt);
          // FIXME: webpack use `walk_statement` here
          this.walk_block_statement(stmt);
        }
        BlockStmtOrExpr::Expr(expr) => this.walk_expression(expr),
      }
    })
  }

  fn walk_expressions<'a, I>(&mut self, expressions: I)
  where
    I: Iterator<Item = &'a Expr>,
  {
    for expr in expressions {
      self.walk_expression(expr)
    }
  }

  fn walk_array_expression(&mut self, expr: &ArrayLit) {
    expr
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.walk_expression(&ele.expr))
  }

  fn walk_nested_statement(&mut self, stmt: &Stmt) {
    // TODO: self.prev_statement = undefined;
    self.walk_statement(stmt);
  }

  fn walk_do_while_statement(&mut self, stmt: &DoWhileStmt) {
    self.walk_nested_statement(&stmt.body);
    self.walk_expression(&stmt.test);
  }

  fn walk_block_statement(&mut self, stmt: &BlockStmt) {
    self.in_block_scope(|this| {
      this.block_pre_walk_statements(&stmt.stmts);
      this.walk_statements(&stmt.stmts);
    })
  }

  fn walk_function_declaration(&mut self, decl: &FnDecl) {
    self.in_function_scope(
      true,
      decl
        .function
        .params
        .iter()
        .map(|param| Cow::Borrowed(&param.pat)),
      |this| {
        this.walk_function(&decl.function);
      },
    )
  }

  fn walk_function(&mut self, f: &Function) {
    for param in &f.params {
      self.walk_pattern(&param.pat)
    }
    if let Some(body) = &f.body {
      self.detect_mode(&body.stmts);
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_block_statement(body);
      // FIXME: webpack use `walk_statement` here
      self.walk_block_statement(body);
    }
  }

  fn walk_function_expression(&mut self, expr: &FnExpr) {
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

    self.in_function_scope(true, scope_params.into_iter(), |this| {
      this.walk_function(&expr.function);
    });
  }

  fn walk_pattern(&mut self, pat: &Pat) {
    match pat {
      Pat::Array(array) => self.walk_array_pattern(array),
      Pat::Assign(assign) => self.walk_assignment_pattern(assign),
      Pat::Object(obj) => self.walk_object_pattern(obj),
      Pat::Rest(rest) => self.walk_rest_element(rest),
      _ => (),
    }
  }

  fn walk_rest_element(&mut self, rest: &RestPat) {
    self.walk_pattern(&rest.arg);
  }

  fn walk_object_pattern(&mut self, obj: &ObjectPat) {
    for prop in &obj.props {
      match prop {
        ObjectPatProp::KeyValue(kv) => {
          if kv.key.is_computed() {
            // FIXME: webpack use `walk_expression` here
            self.walk_prop_name(&kv.key);
          }
          self.walk_pattern(&kv.value);
        }
        ObjectPatProp::Assign(assign) => {
          if let Some(value) = &assign.value {
            self.walk_expression(value);
          }
        }
        ObjectPatProp::Rest(rest) => self.walk_rest_element(rest),
      }
    }
  }

  fn walk_assignment_pattern(&mut self, pat: &AssignPat) {
    self.walk_expression(&pat.right);
    self.walk_pattern(&pat.left);
  }

  fn walk_array_pattern(&mut self, pat: &ArrayPat) {
    pat
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.walk_pattern(ele));
  }

  fn walk_class_declaration(&mut self, decl: &ClassDecl) {
    self.walk_class(Some(&decl.ident), &decl.class);
  }

  fn walk_class(&mut self, ident: Option<&Ident>, classy: &Class) {
    if let Some(super_class) = &classy.super_class {
      // TODO: `hooks.class_extends_expression`
      self.walk_expression(super_class);
    }

    let scope_params = if let Some(pat) = ident.map(|ident| warp_ident_to_pat(ident.clone())) {
      vec![Cow::Owned(pat)]
    } else {
      vec![]
    };

    self.in_class_scope(true, scope_params.into_iter(), |this| {
      for class_element in &classy.body {
        // TODO: `hooks.class_body_element`
        match class_element {
          ClassMember::Constructor(ctor) => {
            if ctor.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&ctor.key);
            }
            for prop in &ctor.params {
              match prop {
                ParamOrTsParamProp::Param(param) => this.walk_pattern(&param.pat),
                ParamOrTsParamProp::TsParamProp(_) => unreachable!(),
              }
            }
            // TODO: `hooks.body_value`;
            if let Some(body) = &ctor.body {
              this.walk_block_statement(body);
            }
          }
          ClassMember::Method(method) => {
            if method.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&method.key);
            }
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| Cow::Borrowed(&p.pat)),
              |this| {
                // TODO: `hooks.body_value`;
                if let Some(body) = &method.function.body {
                  this.walk_block_statement(body);
                }
              },
            );
          }
          ClassMember::PrivateMethod(method) => {
            this.walk_identifier(&method.key.id);
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| Cow::Borrowed(&p.pat)),
              |this| {
                // TODO: `hooks.body_value`;
                if let Some(body) = &method.function.body {
                  this.walk_block_statement(body);
                }
              },
            );
          }
          ClassMember::ClassProp(prop) => {
            if prop.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&prop.key);
            }
            if let Some(value) = &prop.value {
              this.walk_expression(value);
            }
          }
          ClassMember::PrivateProp(prop) => {
            this.walk_identifier(&prop.key.id);
            if let Some(value) = &prop.value {
              this.walk_expression(value);
            }
          }
          ClassMember::Empty(_) => {}
          ClassMember::AutoAccessor(_) => {}
          ClassMember::StaticBlock(block) => this.walk_block_statement(&block.body),
          ClassMember::TsIndexSignature(_) => unreachable!(),
        };
      }
    });
  }
}
