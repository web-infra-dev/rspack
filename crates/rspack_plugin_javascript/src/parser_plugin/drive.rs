use swc_core::{atoms::Atom, common::Span};
use swc_experimental_ecma_ast::{
  AssignExpr, AwaitExpr, BinExpr, CallExpr, Callee, ClassMember, CondExpr, Expr, ForOfStmt, Ident,
  IfStmt, ImportDecl, MemberExpr, ModuleDecl, NewExpr, OptChainExpr, Program, UnaryExpr, UnaryOp,
  VarDeclarator,
};

use super::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
use crate::{
  parser_plugin::r#const::is_logic_op,
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ClassDeclOrExpr, DestructuringAssignmentProperty, ExportDefaultDeclaration,
    ExportDefaultExpression, ExportImport, ExportLocal, ExportedVariableInfo, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

pub struct JavaScriptParserPluginDrive {
  plugins: Vec<BoxJavascriptParserPlugin>,
}

impl JavaScriptParserPluginDrive {
  pub fn new(plugins: Vec<BoxJavascriptParserPlugin>) -> Self {
    Self { plugins }
  }
}

impl JavascriptParserPlugin for JavaScriptParserPluginDrive {
  fn top_level_await_expr(&self, parser: &mut JavascriptParser, expr: AwaitExpr) {
    for plugin in &self.plugins {
      // `SyncBailHook` but without return value
      plugin.top_level_await_expr(parser, expr);
    }
  }

  fn top_level_for_of_await_stmt(&self, parser: &mut JavascriptParser, stmt: ForOfStmt) {
    for plugin in &self.plugins {
      // `SyncBailHook` but without return value
      plugin.top_level_for_of_await_stmt(parser, stmt);
    }
  }

  fn program(&self, parser: &mut JavascriptParser, ast: Program) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.program(parser, ast);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.finish(parser);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn block_pre_module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: ModuleDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.block_pre_module_declaration(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, decl: ModuleDecl) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.module_declaration(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call(parser, expr, name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges,
      );
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    assert!(matches!(expr.callee(&parser.ast), Callee::Expr(_)));
    for plugin in &self.plugins {
      let res = plugin.call_member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges,
      );
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn is_pure(&self, parser: &mut JavascriptParser, expr: Expr) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.is_pure(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    callee_members: &[Atom],
    call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member_chain_of_call_member_chain(
        parser,
        member_expr,
        callee_members,
        call_expr,
        members,
        member_ranges,
        for_name,
      );
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    callee_members: &[Atom],
    inner_call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call_member_chain_of_call_member_chain(
        parser,
        call_expr,
        callee_members,
        inner_call_expr,
        members,
        member_ranges,
        for_name,
      );
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    expr: AssignExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.assign(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: AssignExpr,
    members: &[Atom],
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.assign_member_chain(parser, expr, members, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op(&parser.ast) == UnaryOp::TypeOf);
    for plugin in &self.plugins {
      let res = plugin.r#typeof(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: BinExpr,
  ) -> Option<bool> {
    assert!(is_logic_op(expr.op(&parser.ast)));
    for plugin in &self.plugins {
      let res = plugin.expression_logical_operator(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn binary_expression(&self, parser: &mut JavascriptParser, expr: BinExpr) -> Option<bool> {
    assert!(!is_logic_op(expr.op(&parser.ast)));
    for plugin in &self.plugins {
      let res = plugin.binary_expression(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn statement_if(&self, parser: &mut JavascriptParser, expr: IfStmt) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.statement_if(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    expr: VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.declarator(parser, expr, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.new_expression(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn identifier(&self, parser: &mut JavascriptParser, expr: Ident, for_name: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.identifier(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser,
    super_class: Expr,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_extends_expression(parser, super_class, class_decl_or_expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    member: ClassMember,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_body_element(parser, member, class_decl_or_expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser,
    element: ClassMember,
    expr_span: Span,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_body_value(parser, element, expr_span, class_decl_or_expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn this(&self, parser: &mut JavascriptParser, expr: ThisExpr, for_name: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.this(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    for plugin in &self.plugins {
      let res = plugin.evaluate(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_typeof(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate_call_expression_member<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_call_expression_member(parser, property, expr, param.clone());
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_identifier(parser, for_name, start, end);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser,
    expr: Expr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.can_collect_destructuring_assignment_properties(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pattern(&self, parser: &mut JavascriptParser, ident: Ident, for_name: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pattern(parser, ident, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pre_declarator(parser, declarator, declaration);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.can_rename(parser, str);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: Expr, str: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.rename(parser, expr, str);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pre_statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn block_pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.block_pre_statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import_call(
    &self,
    parser: &mut JavascriptParser,
    expr: CallExpr,
    import_then: Option<CallExpr>,
  ) -> Option<bool> {
    assert!(expr.callee(&parser.ast).is_import());
    for plugin in &self.plugins {
      let res = plugin.import_call(parser, expr, import_then);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.meta_property(parser, root_name, span);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &MemberExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.unhandled_expression_member_chain(parser, root_info, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import(
    &self,
    parser: &mut JavascriptParser,
    statement: ImportDecl,
    source: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.import(parser, statement, source);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ImportDecl,
    source: &swc_core::atoms::Atom,
    export_name: Option<&Atom>,
    identifier_name: &Atom,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.import_specifier(parser, statement, source, export_name, identifier_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_import(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_import(parser, statement, source);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export(&self, parser: &mut JavascriptParser, statement: ExportLocal) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export(parser, statement);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
    local_id: Option<&Atom>,
    export_name: Option<&Atom>,
    export_name_span: Option<Span>,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_import_specifier(
        parser,
        statement,
        source,
        local_id,
        export_name,
        export_name_span,
      );
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportLocal,
    local_id: &Atom,
    export_name: &Atom,
    export_name_span: Span,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_specifier(parser, statement, local_id, export_name, export_name_span);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_expression(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportDefaultDeclaration,
    expr: ExportDefaultExpression,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_expression(parser, statement, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn optional_chaining(&self, parser: &mut JavascriptParser, expr: OptChainExpr) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.optional_chaining(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expr: CondExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.expression_conditional_operation(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import_meta_property_in_destructuring(
    &self,
    parser: &mut JavascriptParser,
    property: DestructuringAssignmentProperty,
  ) -> Option<String> {
    for plugin in &self.plugins {
      let res = plugin.import_meta_property_in_destructuring(parser, property);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
