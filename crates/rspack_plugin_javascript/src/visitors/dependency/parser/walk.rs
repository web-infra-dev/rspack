use std::borrow::Cow;

use swc_core::common::Spanned;
use swc_core::ecma::ast::{
  ArrayLit, ArrayPat, ArrowExpr, AssignExpr, AssignPat, AssignTarget, AssignTargetPat, AwaitExpr,
  Param, SimpleAssignTarget,
};
use swc_core::ecma::ast::{BinExpr, BlockStmt, BlockStmtOrExpr, CallExpr, Callee, CatchClause};
use swc_core::ecma::ast::{Class, ClassDecl, ClassExpr, ClassMember, CondExpr, Decl, DefaultDecl};
use swc_core::ecma::ast::{DoWhileStmt, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr};
use swc_core::ecma::ast::{ExprOrSpread, ExprStmt, FnDecl, MemberExpr, MemberProp, VarDeclOrExpr};
use swc_core::ecma::ast::{FnExpr, ForHead, Function, Ident, KeyValueProp};
use swc_core::ecma::ast::{ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt, WithStmt};
use swc_core::ecma::ast::{MetaPropExpr, NamedExport, NewExpr, ObjectLit, OptCall};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, ObjectPat, ObjectPatProp, Stmt, WhileStmt};
use swc_core::ecma::ast::{OptChainExpr, Pat, ThisExpr, UnaryOp};
use swc_core::ecma::ast::{Prop, PropName, PropOrSpread, RestPat, ReturnStmt, SeqExpr, TaggedTpl};
use swc_core::ecma::ast::{SwitchCase, SwitchStmt, Tpl, TryStmt, VarDecl, YieldExpr};
use swc_core::ecma::ast::{ThrowStmt, UnaryExpr, UpdateExpr};

use super::TopLevelScope;
use super::{AllowedMemberTypes, CallHooksName, JavascriptParser, MemberExpressionInfo, RootName};
use crate::parser_plugin::{is_logic_op, JavascriptParserPlugin};
use crate::visitors::scope_info::{FreeName, VariableInfo};

fn warp_ident_to_pat(ident: Ident) -> Pat {
  Pat::Ident(ident.into())
}

impl<'parser> JavascriptParser<'parser> {
  fn in_block_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(&old_definitions);
    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  fn in_class_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_try = false;
    self.in_tagged_template_tag = false;
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
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  fn in_function_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.definitions = self.definitions_db.create_child(&old_definitions);
    self.in_tagged_template_tag = false;
    if has_this {
      self.undefined_variable("this".to_string());
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.to_string());
    });
    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub fn walk_module_declarations(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.walk_module_declaration(statement);
    }
  }

  fn walk_module_declaration(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(m) => {
        self.statement_path.push(m.span().into());
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
        self.prev_statement = self.statement_path.pop();
      }
      ModuleItem::Stmt(s) => self.walk_statement(s),
    }
  }

  fn walk_export_decl(&mut self, expr: &ExportDecl) {
    // FIXME: delete `ExportDecl`
    self.plugin_drive.clone().export_decl(self, expr);

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
    // TODO: delete `export_default_expr`
    self.plugin_drive.clone().export_default_expr(self, expr);
    self.walk_expression(&expr.expr);
  }

  fn walk_export_named_declaration(&mut self, decl: &NamedExport) {
    self.plugin_drive.clone().named_export(self, decl);
    // self.walk_statement(decl)
  }

  fn walk_export_default_declaration(&mut self, decl: &ExportDefaultDecl) {
    self.plugin_drive.clone().export(self, decl);
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
    self.statement_path.push(statement.span().into());
    // TODO: `self.hooks.statement.call`
    let old_last_stmt_is_expr_stmt = self.last_stmt_is_expr_stmt;
    self.stmt_level += 1;

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
      Stmt::Expr(stmt) => {
        self.last_stmt_is_expr_stmt = true;
        self.walk_expression_statement(stmt)
      }
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

    self.last_stmt_is_expr_stmt = old_last_stmt_is_expr_stmt;
    self.stmt_level -= 1;
    self.prev_statement = self.statement_path.pop();
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
      let prev = this.prev_statement.clone();
      this.block_pre_walk_statements(&catch_clause.body.stmts);
      this.prev_statement = prev;
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
          let prev = this.prev_statement.clone();
          this.block_pre_walk_statements(&case.cons);
          this.prev_statement = prev;
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
    if let Some(result) = self.plugin_drive.clone().statement_if(self, stmt) {
      if result {
        self.walk_nested_statement(&stmt.cons);
        // TODO: adapt `InnerGraphPlugin` to `ParserPlugin`
        self.path_ignored_spans.push(stmt.alt.span());
      } else if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt);
        // TODO: adapt `InnerGraphPlugin` to `ParserPlugin`
        self.path_ignored_spans.push(stmt.cons.span());
      }
    } else {
      self.walk_expression(&stmt.test);
      self.walk_nested_statement(&stmt.cons);
      if let Some(alt) = &stmt.alt {
        self.walk_nested_statement(alt);
      }
    }
  }

  fn walk_for_statement(&mut self, stmt: &ForStmt) {
    self.in_block_scope(|this| {
      if let Some(init) = &stmt.init {
        match init {
          VarDeclOrExpr::VarDecl(decl) => {
            this.block_pre_walk_variable_declaration(decl);
            this.prev_statement = None;
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
        let prev = this.prev_statement.clone();
        this.block_pre_walk_statements(&body.stmts);
        this.prev_statement = prev;
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
        let prev = this.prev_statement.clone();
        this.block_pre_walk_statements(&body.stmts);
        this.prev_statement = prev;
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
        let prev = this.prev_statement.clone();
        this.block_pre_walk_statements(&body.stmts);
        this.prev_statement = prev;
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
      if let Some(init) = declarator.init.as_ref()
        && let Some(renamed_identifier) = self.get_rename_identifier(init)
        && let Some(ident) = declarator.name.as_ident()
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
            self.set_variable(ident.sym.to_string(), renamed_identifier);
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
      if let Some(expr_info) =
        self.get_member_expression_info_from_expr(&expr.arg, AllowedMemberTypes::Expression)
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
    }
    self.walk_expression(&expr.arg)
  }

  fn walk_this_expression(&mut self, expr: &ThisExpr) {
    // TODO: `this.hooks.call_hooks_for_names`
    self.plugin_drive.clone().this(self, expr);
  }

  pub(crate) fn walk_template_expression(&mut self, expr: &Tpl) {
    let exprs = expr.exprs.iter().map(|expr| &**expr);
    self.walk_expressions(exprs);
  }

  fn walk_tagged_template_expression(&mut self, expr: &TaggedTpl) {
    self.in_tagged_template_tag = true;
    self.walk_expression(&expr.tag);
    self.in_tagged_template_tag = false;

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
        self.walk_prop_name(&getter.key);
        let was_top_level = self.top_level_scope;
        self.top_level_scope = TopLevelScope::False;
        if let Some(body) = &getter.body {
          self.walk_block_statement(body);
        }
        self.top_level_scope = was_top_level;
      }
      Prop::Setter(setter) => {
        self.walk_prop_name(&setter.key);
        let was_top_level = self.top_level_scope;
        self.top_level_scope = TopLevelScope::False;
        if let Some(body) = &setter.body {
          self.walk_block_statement(body);
        }
        self.top_level_scope = was_top_level;
      }
      Prop::Method(method) => {
        self.walk_prop_name(&method.key);
        let was_top_level = self.top_level_scope;
        self.top_level_scope = TopLevelScope::False;
        self.in_function_scope(
          true,
          method.function.params.iter().map(|p| Cow::Borrowed(&p.pat)),
          |parser| {
            parser.walk_function(&method.function);
          },
        );
        self.top_level_scope = was_top_level;
      }
    }
  }

  fn walk_prop_name(&mut self, prop_name: &PropName) {
    if let Some(computed) = prop_name.as_computed() {
      self.walk_expression(&computed.expr);
    }
  }

  fn walk_new_expression(&mut self, expr: &NewExpr) {
    if let Some(MemberExpressionInfo::Expression(info)) =
      self.get_member_expression_info_from_expr(&expr.callee, AllowedMemberTypes::Expression)
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
    self.walk_expression(&expr.callee);
    if let Some(args) = &expr.args {
      self.walk_expr_or_spread(args);
    }
  }

  fn walk_meta_property(&mut self, expr: &MetaPropExpr) {
    let Some(root_name) = expr.get_root_name() else {
      unreachable!()
    };
    self
      .plugin_drive
      .clone()
      .meta_property(self, &root_name, expr.span);
  }

  fn walk_conditional_expression(&mut self, expr: &CondExpr) {
    let result = self
      .plugin_drive
      .clone()
      .expression_conditional_operation(self, expr);

    if let Some(result) = result {
      if result {
        self.walk_expression(&expr.cons);
      } else {
        self.walk_expression(&expr.alt);
      }
    } else {
      self.walk_expression(&expr.test);
      self.walk_expression(&expr.cons);
      self.walk_expression(&expr.alt);
    }
  }

  fn walk_class_expression(&mut self, expr: &ClassExpr) {
    self.walk_class(expr.ident.as_ref(), &expr.class);
  }

  fn walk_chain_expression(&mut self, expr: &OptChainExpr) {
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

  fn walk_member_expression(&mut self, expr: &MemberExpr) {
    // println!("{:#?}", expr);
    if let Some(expr_info) = self.get_member_expression_info(expr, AllowedMemberTypes::all()) {
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
              this
                .plugin_drive
                .clone()
                .member_chain_of_call_member_chain(this, expr, for_name)
            })
            .unwrap_or_default()
          {
            return;
          }
          self.walk_call_expression(&expr_info.call);
          return;
        }
      }
    }
    self.member_expr_in_optional_chain = false;
    self.walk_expression(&expr.obj);
    if let MemberProp::Computed(computed) = &expr.prop {
      self.walk_expression(&computed.expr)
    }
  }

  fn walk_member_expression_with_expression_name<F>(
    &mut self,
    expr: &MemberExpr,
    name: &str,
    on_unhandled: Option<F>,
  ) where
    F: FnOnce(&mut Self) -> Option<bool>,
  {
    if let Some(member) = expr.obj.as_member()
      && let Some(len) = member_prop_len(&expr.prop)
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
      self.walk_expression(&expr.obj);
    } else if let Some(on_unhandled) = on_unhandled
      && !on_unhandled(self).unwrap_or_default()
    {
      self.walk_expression(&expr.obj);
    }

    if let MemberProp::Computed(computed) = &expr.prop {
      self.walk_expression(&computed.expr)
    }
  }

  fn walk_opt_call(&mut self, expr: &OptCall) {
    self.walk_call_expression(&CallExpr {
      span: expr.span,
      callee: Callee::Expr(expr.callee.clone()),
      args: expr.args.clone(),
      type_args: None,
    })
  }

  /// Walk IIFE function
  ///
  /// # Panics
  /// Either `Params` of `expr` or `params` passed in should be `BindingIdent`.
  fn _walk_iife<'a>(
    &mut self,
    expr: &'a Expr,
    params: impl Iterator<Item = &'a Expr>,
    current_this: Option<&'a Expr>,
  ) {
    let mut fn_params = vec![];
    let mut scope_params = vec![];
    if let Some(expr) = expr.as_fn_expr() {
      for param in &expr.function.params {
        let ident = param.pat.as_ident().expect("should be a `BindingIdent`");
        fn_params.push(ident);
        if get_variable_info(self, &Expr::Ident(ident.id.clone())).is_none() {
          scope_params.push(Cow::Borrowed(&param.pat));
        }
      }
    } else if let Some(expr) = expr.as_arrow() {
      for param in &expr.params {
        let ident = param.as_ident().expect("should be a `BindingIdent`");
        fn_params.push(ident);
        if get_variable_info(self, &Expr::Ident(ident.id.clone())).is_none() {
          scope_params.push(Cow::Borrowed(param));
        }
      }
    };
    let variable_info_for_args = params
      .map(|param| get_variable_name(self, param))
      .collect::<Vec<_>>();
    if let Some(expr) = expr.as_fn_expr() {
      if let Some(ident) = &expr.ident {
        scope_params.push(Cow::Owned(Pat::Ident(ident.clone().into())));
      }
    }

    let was_top_level_scope = self.top_level_scope;
    self.top_level_scope =
      if !matches!(was_top_level_scope, TopLevelScope::False) && expr.as_arrow().is_some() {
        TopLevelScope::ArrowFunction
      } else {
        TopLevelScope::False
      };

    let rename_this = current_this.and_then(|this| get_variable_name(self, this));
    self.in_function_scope(true, scope_params.into_iter(), |parser| {
      if let Some(this) = rename_this
        && matches!(expr, Expr::Fn(_))
      {
        parser.set_variable("this".to_string(), this)
      }
      for (variable_info, param) in variable_info_for_args.into_iter().zip(fn_params) {
        let Some(variable_info) = variable_info else {
          continue;
        };
        parser.set_variable(param.sym.to_string(), variable_info);
      }

      if let Some(expr) = expr.as_fn_expr() {
        if let Some(stmt) = &expr.function.body {
          parser.detect_mode(&stmt.stmts);
          let prev = parser.prev_statement.clone();
          // FIXME: webpack use `pre_walk_statement` here
          parser.pre_walk_block_statement(stmt);
          parser.prev_statement = prev;
          // FIXME: webpack use `walk_statement` here
          parser.walk_block_statement(stmt);
        }
      } else if let Some(expr) = expr.as_arrow() {
        match &*expr.body {
          BlockStmtOrExpr::BlockStmt(stmt) => {
            parser.detect_mode(&stmt.stmts);
            let prev = parser.prev_statement.clone();
            // FIXME: webpack use `pre_walk_statement` here
            parser.pre_walk_block_statement(stmt);
            parser.prev_statement = prev;
            // FIXME: webpack use `walk_statement` here
            parser.walk_block_statement(stmt);
          }
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        }
      }
    });
    self.top_level_scope = was_top_level_scope;
  }

  fn walk_call_expression(&mut self, expr: &CallExpr) {
    self.enter_call += 1;

    fn is_simple_function(params: &[Param]) -> bool {
      params.iter().all(|p| matches!(p.pat, Pat::Ident(_)))
    }

    // FIXME: should align to webpack
    match &expr.callee {
      Callee::Expr(callee) => {
        if let Expr::Member(member_expr) = &**callee
          && let Expr::Paren(paren_expr) = &*member_expr.obj
          && let Expr::Fn(fn_expr) = &*paren_expr.expr
          && let MemberProp::Ident(ident) = &member_expr.prop
          && (ident.sym == "call" || ident.sym == "bind")
          && !expr.args.is_empty()
          && is_simple_function(&fn_expr.function.params)
        {
          // (function(…) { }).call(…)
          let mut params = expr.args.iter().map(|arg| &*arg.expr);
          let this = params.next();
          self._walk_iife(&paren_expr.expr, params, this)
        } else if let Expr::Member(member_expr) = &**callee
          && let Expr::Fn(fn_expr) = &*member_expr.obj
          && let MemberProp::Ident(ident) = &member_expr.prop
          && (ident.sym == "call" || ident.sym == "bind")
          && !expr.args.is_empty()
          && is_simple_function(&fn_expr.function.params)
        {
          // (function(…) { }.call(…))
          let mut params = expr.args.iter().map(|arg| &*arg.expr);
          let this = params.next();
          self._walk_iife(&member_expr.obj, params, this)
        } else if let Expr::Paren(paren_expr) = &**callee
          && let Expr::Fn(fn_expr) = &*paren_expr.expr
          && is_simple_function(&fn_expr.function.params)
        {
          // (function(…) { })(…)
          self._walk_iife(
            &paren_expr.expr,
            expr.args.iter().map(|arg| &*arg.expr),
            None,
          )
        } else if let Expr::Fn(fn_expr) = &**callee
          && is_simple_function(&fn_expr.function.params)
        {
          // (function(…) { }(…))
          self._walk_iife(callee, expr.args.iter().map(|arg| &*arg.expr), None)
        } else {
          if let Expr::Member(member) = &**callee
            && let Some(MemberExpressionInfo::Call(expr_info)) =
              self.get_member_expression_info(member, AllowedMemberTypes::CallExpression)
            && expr_info
              .root_info
              .call_hooks_name(self, |this, for_name| {
                this
                  .plugin_drive
                  .clone()
                  .call_member_chain_of_call_member_chain(this, expr, for_name)
              })
              .unwrap_or_default()
          {
            self.enter_call -= 1;
            return;
          }
          let evaluated_callee = self.evaluate_expression(callee);
          if evaluated_callee.is_identifier() {
            let members = evaluated_callee
              .members()
              .map(Cow::Borrowed)
              .unwrap_or_else(|| Cow::Owned(Vec::new()));
            let members_optionals = evaluated_callee
              .members_optionals()
              .map(Cow::Borrowed)
              .unwrap_or_else(|| Cow::Owned(members.iter().map(|_| false).collect::<Vec<_>>()));
            let member_ranges = evaluated_callee
              .member_ranges()
              .map(Cow::Borrowed)
              .unwrap_or_else(|| Cow::Owned(Vec::new()));
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
              self.enter_call -= 1;
              return;
            }

            if drive
              .call(self, expr, evaluated_callee.identifier())
              .unwrap_or_default()
            {
              /* result2 */
              self.enter_call -= 1;
              return;
            }
          }

          if let Some(member) = callee.as_member() {
            self.walk_expression(&member.obj);
            if let Some(computed) = member.prop.as_computed() {
              self.walk_expression(&computed.expr);
            }
          } else {
            self.walk_expression(callee);
          }
        }
      }
      Callee::Import(_) => {
        // In webpack this is walkImportExpression, import() is a ImportExpression instead of CallExpression with Callee::Import
        if self
          .plugin_drive
          .clone()
          .import_call(self, expr)
          .unwrap_or_default()
        {
          self.enter_call -= 1;
          return;
        }
      }
      Callee::Super(_) => {} // Do nothing about super, same as webpack
    }

    self.walk_expr_or_spread(&expr.args);
    self.enter_call -= 1;
  }

  pub fn walk_expr_or_spread(&mut self, args: &Vec<ExprOrSpread>) {
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
        } else {
          // TODO: adapt `InnerGraphPlugin` to `ParserPlugin`
          self.path_ignored_spans.push(expr.right.span());
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
    if matches!(self.top_level_scope, TopLevelScope::Top) {
      self.plugin_drive.clone().top_level_await_expr(self, expr);
    }
    self.walk_expression(&expr.arg);
  }

  fn walk_identifier(&mut self, identifier: &Ident) {
    identifier.sym.call_hooks_name(self, |this, for_name| {
      this
        .plugin_drive
        .clone()
        .identifier(this, identifier, for_name)
    });
  }

  fn get_rename_identifier(&mut self, expr: &Expr) -> Option<String> {
    let result = self.evaluate_expression(expr);
    result
      .is_identifier()
      .then(|| result.identifier().to_string())
  }

  fn walk_assignment_expression(&mut self, expr: &AssignExpr) {
    // FIXME: should align to webpack
    if self
      .plugin_drive
      .clone()
      .assign(self, expr)
      .unwrap_or_default()
    {
      // empty
    } else if let Some(ident) = expr.left.as_ident() {
      if let Some(rename_identifier) = self.get_rename_identifier(&expr.right)
        && let drive = self.plugin_drive.clone()
        && rename_identifier
          .call_hooks_name(self, |this, for_name| drive.can_rename(this, for_name))
          .unwrap_or_default()
      {
        if !rename_identifier
          .call_hooks_name(self, |this, for_name| {
            drive.rename(this, &expr.right, for_name)
          })
          .unwrap_or_default()
        {
          let variable = self
            .get_variable_info(&rename_identifier)
            .map(|info| info.free_name.as_ref())
            .and_then(|free_name| free_name)
            .and_then(|free_name| match free_name {
              FreeName::String(s) => Some(s.to_string()),
              FreeName::True => None,
            })
            .unwrap_or(rename_identifier);
          self.set_variable(ident.sym.to_string(), variable);
        }
        return;
      }
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
      self.enter_assign_target_pattern(Cow::Borrowed(pat), |this, ident| {
        // TODO: if (!this.callHooksForName(this.hooks.assign, name, expression)) {
        this.define_variable(ident.sym.to_string());
      });
      self.walk_assign_target_pattern(pat);
    } else {
      self.walk_expression(&expr.right);
      match &expr.left {
        AssignTarget::Simple(simple) => self.walk_simple_assign_target(simple),
        AssignTarget::Pat(pat) => self.walk_assign_target_pattern(pat),
      }
    }
    // TODO:
    // else if let Some(member) = expr.left.as_expr().and_then(|expr| expr.as_member()) {
    // }
  }

  fn walk_arrow_function_expression(&mut self, expr: &ArrowExpr) {
    let was_top_level_scope = self.top_level_scope;
    if !matches!(was_top_level_scope, TopLevelScope::False) {
      self.top_level_scope = TopLevelScope::ArrowFunction;
    }
    self.in_function_scope(false, expr.params.iter().map(Cow::Borrowed), |this| {
      for param in &expr.params {
        this.walk_pattern(param)
      }
      match &*expr.body {
        BlockStmtOrExpr::BlockStmt(stmt) => {
          this.detect_mode(&stmt.stmts);
          let prev = this.prev_statement.clone();
          // FIXME: webpack use `pre_walk_statement` here
          this.pre_walk_block_statement(stmt);
          this.prev_statement = prev;
          // FIXME: webpack use `walk_statement` here
          this.walk_block_statement(stmt);
        }
        BlockStmtOrExpr::Expr(expr) => this.walk_expression(expr),
      }
    });
    self.top_level_scope = was_top_level_scope;
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
    self.prev_statement = None;
    self.walk_statement(stmt);
  }

  fn walk_do_while_statement(&mut self, stmt: &DoWhileStmt) {
    self.walk_nested_statement(&stmt.body);
    self.walk_expression(&stmt.test);
  }

  fn walk_block_statement(&mut self, stmt: &BlockStmt) {
    self.in_block_scope(|this| {
      let prev = this.prev_statement.clone();
      this.block_pre_walk_statements(&stmt.stmts);
      this.prev_statement = prev;
      this.walk_statements(&stmt.stmts);
    })
  }

  fn walk_function_declaration(&mut self, decl: &FnDecl) {
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
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
    );
    self.top_level_scope = was_top_level;
  }

  fn walk_function(&mut self, f: &Function) {
    for param in &f.params {
      self.walk_pattern(&param.pat)
    }
    if let Some(body) = &f.body {
      self.detect_mode(&body.stmts);
      let prev = self.prev_statement.clone();
      // FIXME: webpack use `pre_walk_statement` here
      self.pre_walk_block_statement(body);
      self.prev_statement = prev;
      // FIXME: webpack use `walk_statement` here
      self.walk_block_statement(body);
    }
  }

  fn walk_function_expression(&mut self, expr: &FnExpr) {
    let was_top_level = self.top_level_scope;
    self.top_level_scope = TopLevelScope::False;
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
    self.top_level_scope = was_top_level;
  }

  fn walk_pattern(&mut self, pat: &Pat) {
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

  fn walk_simple_assign_target(&mut self, target: &SimpleAssignTarget) {
    match target {
      SimpleAssignTarget::Ident(ident) => self.walk_identifier(ident),
      SimpleAssignTarget::Member(member) => self.walk_member_expression(member),
      SimpleAssignTarget::Paren(expr) => self.walk_expression(&expr.expr),
      SimpleAssignTarget::OptChain(expr) => self.walk_chain_expression(expr),
      SimpleAssignTarget::SuperProp(_) => (),
      SimpleAssignTarget::TsAs(_)
      | SimpleAssignTarget::TsSatisfies(_)
      | SimpleAssignTarget::TsNonNull(_)
      | SimpleAssignTarget::TsTypeAssertion(_)
      | SimpleAssignTarget::TsInstantiation(_) => (),
      SimpleAssignTarget::Invalid(_) => (),
    }
  }

  fn walk_assign_target_pattern(&mut self, pat: &AssignTargetPat) {
    match pat {
      AssignTargetPat::Array(array) => self.walk_array_pattern(array),
      AssignTargetPat::Object(obj) => self.walk_object_pattern(obj),
      AssignTargetPat::Invalid(_) => (),
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

            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;

            let params = ctor.params.iter().map(|p| {
              let p = p.as_param().expect("should only contain param");
              Cow::Borrowed(&p.pat)
            });
            this.in_function_scope(true, params.clone(), |this| {
              for param in params {
                this.walk_pattern(&param)
              }

              // TODO: `hooks.body_value`;
              if let Some(body) = &ctor.body {
                this.detect_mode(&body.stmts);
                let prev = this.prev_statement.clone();
                this.pre_walk_block_statement(body);
                this.prev_statement = prev;
                this.walk_block_statement(body);
              }
            });

            this.top_level_scope = was_top_level;
          }
          ClassMember::Method(method) => {
            if method.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&method.key);
            }
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| Cow::Borrowed(&p.pat)),
              |this| this.walk_function(&method.function),
            );
            this.top_level_scope = was_top_level;
          }
          ClassMember::PrivateMethod(method) => {
            // method.key is always not computed in private method, so we don't need to walk it
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            this.in_function_scope(
              true,
              method.function.params.iter().map(|p| Cow::Borrowed(&p.pat)),
              |this| this.walk_function(&method.function),
            );
            this.top_level_scope = was_top_level;
          }
          ClassMember::ClassProp(prop) => {
            if prop.key.is_computed() {
              // FIXME: webpack use `walk_expression` here
              this.walk_prop_name(&prop.key);
            }
            if let Some(value) = &prop.value {
              let was_top_level = this.top_level_scope;
              this.top_level_scope = TopLevelScope::False;
              this.walk_expression(value);
              this.top_level_scope = was_top_level;
            }
          }
          ClassMember::PrivateProp(prop) => {
            // prop.key is always not computed in private prop, so we don't need to walk it
            if let Some(value) = &prop.value {
              let was_top_level = this.top_level_scope;
              this.top_level_scope = TopLevelScope::False;
              this.walk_expression(value);
              this.top_level_scope = was_top_level;
            }
          }
          ClassMember::StaticBlock(block) => {
            let was_top_level = this.top_level_scope;
            this.top_level_scope = TopLevelScope::False;
            this.walk_block_statement(&block.body);
            this.top_level_scope = was_top_level;
          }
          ClassMember::Empty(_) => {}
          ClassMember::AutoAccessor(_) => {}
          ClassMember::TsIndexSignature(_) => unreachable!(),
        };
      }
    });
  }
}

fn member_prop_len(member_prop: &MemberProp) -> Option<usize> {
  match member_prop {
    MemberProp::Ident(ident) => Some(ident.sym.len()),
    MemberProp::PrivateName(name) => Some(name.id.sym.len() + 1),
    MemberProp::Computed(_) => None,
  }
}

fn get_variable_info<'p>(
  parser: &'p mut JavascriptParser,
  expr: &Expr,
) -> Option<&'p VariableInfo> {
  if let Some(rename_identifier) = parser.get_rename_identifier(expr)
    && let drive = parser.plugin_drive.clone()
    && rename_identifier
      .call_hooks_name(parser, |this, for_name| drive.can_rename(this, for_name))
      .unwrap_or_default()
    && !rename_identifier
      .call_hooks_name(parser, |this, for_name| drive.rename(this, expr, for_name))
      .unwrap_or_default()
  {
    return parser.get_variable_info(&rename_identifier);
  }
  None
}

fn get_variable_name(parser: &mut JavascriptParser, expr: &Expr) -> Option<String> {
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
      .map(|info| info.free_name.as_ref())
      .and_then(|free_name| free_name)
      .and_then(|free_name| match free_name {
        FreeName::String(s) => Some(s.to_string()),
        FreeName::True => None,
      })
      .unwrap_or(rename_identifier);
    return Some(variable);
  }
  None
}
