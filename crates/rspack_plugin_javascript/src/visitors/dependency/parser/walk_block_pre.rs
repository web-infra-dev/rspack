use swc_atoms::Atom;
use swc_next_ecma_ast::{ExprData, GetSpan, Stmt, StmtData, TypedSubRange};

use super::{
  JavascriptParser,
  estree::{
    ExportDefaultDeclaration, ExportDefaultExpression, ExportLocal, ExportNamedDeclaration,
    MaybeNamedClassDecl, Statement,
  },
};
use crate::{
  JS_DEFAULT_KEYWORD,
  parser_plugin::JavascriptParserPlugin,
  visitors::{VariableDeclaration, VariableDeclarationKind},
};

impl JavascriptParser<'_> {
  pub fn block_pre_walk_module_items(&mut self, statements: TypedSubRange<Stmt>) {
    let ast = self.ast.ast;
    for id in statements.iter() {
      let statement = ast.get_node_in_sub_range(id);
      self.block_pre_walk_module_item(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: TypedSubRange<Stmt>) {
    let ast = self.ast.ast;
    for id in statements.iter() {
      let statement = ast.get_node_in_sub_range(id);
      self.block_pre_walk_statement(Statement::from_stmt(ast, statement));
    }
  }

  pub fn block_pre_walk_module_item(&mut self, statement: Stmt) {
    let ast = self.ast.ast;
    match ast.stmt_data(statement) {
      StmtData::ImportDeclaration(_)
      | StmtData::ExportAllDeclaration(_)
      | StmtData::ExportNamedDeclaration(_)
      | StmtData::ExportDefaultDeclaration(_) => {
        let drive = self.plugin_drive.clone();
        self.enter_statement(
          statement.span(ast),
          statement,
          |parser, node| {
            drive
              .block_pre_module_declaration(parser, node)
              .unwrap_or_default()
          },
          |parser, node| match parser.ast.ast.stmt_data(node) {
            StmtData::ExportNamedDeclaration(declaration) => {
              parser.block_pre_walk_export_named_declaration(ExportNamedDeclaration(declaration))
            }
            StmtData::ExportDefaultDeclaration(declaration) => parser
              .block_pre_walk_export_default_declaration(ExportDefaultDeclaration(declaration)),
            _ => (),
          },
        );
      }
      _ => self.block_pre_walk_statement(Statement::from_stmt(ast, statement)),
    }
  }

  pub fn block_pre_walk_statement(&mut self, statement: Statement) {
    let drive = self.plugin_drive.clone();
    self.enter_statement(
      statement.span(self.ast.ast),
      statement,
      |parser, node| drive.block_pre_statement(parser, node).unwrap_or_default(),
      |parser, node| match node {
        Statement::Class(declaration) => parser.block_pre_walk_class_declaration(declaration),
        Statement::Var(declaration) => parser.block_pre_walk_variable_declaration(declaration),
        Statement::Expr(expression) => parser.block_pre_walk_expression_statement(expression),
        _ => (),
      },
    );
  }

  fn block_pre_walk_expression_statement(
    &mut self,
    statement: swc_next_ecma_ast::ExpressionStatement,
  ) {
    let expression = statement.expression(self.ast.ast);
    if let ExprData::AssignmentExpression(assignment) = self.ast.ast.expr_data(expression) {
      self.pre_walk_assignment_expression(assignment);
    }
  }

  pub(super) fn block_pre_walk_variable_declaration(&mut self, declaration: VariableDeclaration) {
    if declaration.kind(self.ast.ast) != VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(declaration);
    }
  }

  fn block_pre_walk_class_declaration(&mut self, declaration: MaybeNamedClassDecl) {
    if let Some(identifier) = declaration.ident(self.ast.ast) {
      self.define_variable(Atom::from(
        self.ast.ast.get_utf8(identifier.name(self.ast.ast)),
      ));
    }
  }

  fn block_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    let ast = self.ast.ast;
    if export.source(ast).is_some() {
      return;
    }
    let drive = self.plugin_drive.clone();
    drive.export(self, ExportLocal::Named(export));
    if let Some(declaration) = export.0.declaration(ast) {
      let statement = Statement::from_stmt(ast, Stmt::Declaration(declaration));
      let prev = self.prev_statement;
      self.pre_walk_statement(statement);
      self.prev_statement = prev;
      self.block_pre_walk_statement(statement);
      self.enter_declaration(declaration, |parser, identifier| {
        let ast = parser.ast.ast;
        let name = Atom::from(ast.get_utf8(identifier.name(ast)));
        drive.export_specifier(
          parser,
          ExportLocal::Named(export),
          &name,
          &name,
          identifier.span(ast),
        );
      });
    } else {
      for (local, exported, span) in export.named_export_specifiers(ast) {
        drive.export_specifier(self, ExportLocal::Named(export), &local, &exported, span);
      }
    }
  }

  fn block_pre_walk_export_default_declaration(&mut self, export: ExportDefaultDeclaration) {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    drive.export(self, ExportLocal::Default(export));
    let expression = export.expression(ast);
    match expression {
      ExportDefaultExpression::ClassDecl(class) => {
        let statement = Statement::Class(MaybeNamedClassDecl(class));
        let prev = self.prev_statement;
        self.pre_walk_statement(statement);
        self.prev_statement = prev;
        self.block_pre_walk_statement(statement);
        if let Some(identifier) = class.id(ast) {
          drive.export_specifier(
            self,
            ExportLocal::Default(export),
            &Atom::from(ast.get_utf8(identifier.name(ast))),
            &JS_DEFAULT_KEYWORD,
            identifier.span(ast),
          );
        } else {
          drive.export_expression(self, export, expression);
        }
      }
      ExportDefaultExpression::FnDecl(function) => {
        let statement = Statement::Fn(super::estree::MaybeNamedFunctionDecl(function));
        let prev = self.prev_statement;
        self.pre_walk_statement(statement);
        self.prev_statement = prev;
        self.block_pre_walk_statement(statement);
        if let Some(identifier) = function.id(ast) {
          drive.export_specifier(
            self,
            ExportLocal::Default(export),
            &Atom::from(ast.get_utf8(identifier.name(ast))),
            &JS_DEFAULT_KEYWORD,
            identifier.span(ast),
          );
        } else {
          drive.export_expression(self, export, expression);
        }
      }
      ExportDefaultExpression::Expr(_) | ExportDefaultExpression::Other(_) => {
        drive.export_expression(self, export, expression);
      }
    }
  }
}
