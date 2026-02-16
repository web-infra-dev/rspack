use swc_experimental_ecma_ast::{
  DefaultDecl, ExportSpecifier, ExprStmt, ModuleDecl, ModuleItem, Spanned, Stmt, TypedSubRange,
};

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
  visitors::{MaybeNamedFunctionDecl, VariableDeclaration, VariableDeclarationKind},
};

impl JavascriptParser<'_> {
  pub fn block_pre_walk_module_items(&mut self, statements: TypedSubRange<ModuleItem>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.block_pre_walk_module_item(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: TypedSubRange<Stmt>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.block_pre_walk_statement(Statement::from_stmt(statement, &self.ast));
    }
  }

  pub fn block_pre_walk_module_item(&mut self, statement: ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        self.enter_statement(
          statement,
          |parser, _| {
            parser
              .plugin_drive
              .clone()
              .block_pre_module_declaration(parser, decl)
              .unwrap_or_default()
          },
          |parser, _| {
            match decl {
              ModuleDecl::Import(_) => {}
              ModuleDecl::ExportAll(_) => {}
              ModuleDecl::ExportNamed(decl) => {
                let is_named_namespace_export = decl.specifiers(&parser.ast).len() == 1
                  && matches!(
                    decl
                      .specifiers(&parser.ast)
                      .first()
                      .map(|n| parser.ast.get_node_in_sub_range(n)),
                    Some(ExportSpecifier::Namespace(_))
                  );
                if !is_named_namespace_export {
                  parser.block_pre_walk_export_named_declaration(
                    ExportNamedDeclaration::Specifiers(decl),
                  )
                }
              }
              ModuleDecl::ExportDecl(decl) => {
                parser.block_pre_walk_export_named_declaration(ExportNamedDeclaration::Decl(decl))
              }
              ModuleDecl::ExportDefaultDecl(decl) => parser
                .block_pre_walk_export_default_declaration(ExportDefaultDeclaration::Decl(decl)),
              ModuleDecl::ExportDefaultExpr(expr) => parser
                .block_pre_walk_export_default_declaration(ExportDefaultDeclaration::Expr(expr)),
            };
          },
        );
      }
      ModuleItem::Stmt(stmt) => {
        self.block_pre_walk_statement(Statement::from_stmt(stmt, &self.ast))
      }
    }
  }

  pub fn block_pre_walk_statement(&mut self, stmt: Statement) {
    self.enter_statement(
      stmt,
      |parser, _| {
        parser
          .plugin_drive
          .clone()
          .block_pre_statement(parser, stmt)
          .unwrap_or_default()
      },
      |parser, _| match stmt {
        Statement::Class(decl) => parser.block_pre_walk_class_declaration(decl),
        Statement::Var(decl) => parser.block_pre_walk_variable_declaration(decl),
        Statement::Expr(expr) => parser.block_pre_walk_expression_statement(expr),
        _ => (),
      },
    );
  }

  fn block_pre_walk_expression_statement(&mut self, stmt: ExprStmt) {
    if let Some(assign) = stmt.expr(&self.ast).as_assign() {
      self.pre_walk_assignment_expression(assign)
    }
  }

  pub(super) fn block_pre_walk_variable_declaration(&mut self, decl: VariableDeclaration) {
    if decl.kind(&self.ast) != VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(decl);
    }
  }

  fn block_pre_walk_class_declaration(&mut self, decl: MaybeNamedClassDecl) {
    if let Some(ident) = decl.ident() {
      self.define_variable(self.ast.get_atom(ident.sym(&self.ast)))
    }
  }

  fn block_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    if export.source(&self.ast).is_some() {
      return;
    }
    self
      .plugin_drive
      .clone()
      .export(self, ExportLocal::Named(export));
    match export {
      ExportNamedDeclaration::Decl(decl) => {
        let prev = self.prev_statement;
        self.pre_walk_statement(Statement::from_decl(decl.decl(&self.ast).into(), &self.ast));
        self.prev_statement = prev;
        self.block_pre_walk_statement(Statement::from_decl(decl.decl(&self.ast), &self.ast));
        self.enter_declaration(decl.decl(&self.ast), |parser, def| {
          parser.plugin_drive.clone().export_specifier(
            parser,
            ExportLocal::Named(export),
            &parser.ast.get_atom(def.sym(&parser.ast)),
            &parser.ast.get_atom(def.sym(&parser.ast)),
            def.span(&parser.ast),
          );
        });
      }
      ExportNamedDeclaration::Specifiers(named) => {
        ExportNamedDeclaration::named_export_specifiers(
          self,
          named,
          |this, local_id, exported_name, exported_name_span| {
            if named.src(&this.ast).is_none() {
              this.plugin_drive.clone().export_specifier(
                this,
                ExportLocal::Named(export),
                &local_id,
                &exported_name,
                exported_name_span,
              );
            }
          },
        );
      }
    }
  }

  fn block_pre_walk_export_default_declaration(&mut self, export: ExportDefaultDeclaration) {
    self
      .plugin_drive
      .clone()
      .export(self, ExportLocal::Default(export));
    match export {
      ExportDefaultDeclaration::Decl(decl) => {
        match decl.decl(&self.ast) {
          DefaultDecl::Class(c) => {
            let stmt = Statement::Class(MaybeNamedClassDecl::from_class_expr(c, &self.ast));
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = c.ident(&self.ast) {
              self.plugin_drive.clone().export_specifier(
                self,
                ExportLocal::Default(export),
                &self.ast.get_atom(ident.sym(&self.ast)),
                &JS_DEFAULT_KEYWORD,
                ident.span(&self.ast),
              );
            } else {
              self.plugin_drive.clone().export_expression(
                self,
                export,
                ExportDefaultExpression::ClassDecl(c),
              );
            }
          }
          DefaultDecl::Fn(f) => {
            let stmt = Statement::Fn(MaybeNamedFunctionDecl::from_fn_expr(f, &self.ast));
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = f.ident(&self.ast) {
              self.plugin_drive.clone().export_specifier(
                self,
                ExportLocal::Default(export),
                &self.ast.get_atom(ident.sym(&self.ast)),
                &JS_DEFAULT_KEYWORD,
                ident.span(&self.ast),
              );
            } else {
              self.plugin_drive.clone().export_expression(
                self,
                export,
                ExportDefaultExpression::FnDecl(f),
              );
            }
          }
        };
      }
      ExportDefaultDeclaration::Expr(expr) => {
        // Webpack call exportExpression in walk (legacy code maybe)
        // We move it to block_pre_walk for consistent with other export related hook
        self.plugin_drive.clone().export_expression(
          self,
          export,
          ExportDefaultExpression::Expr(expr.expr(&self.ast)),
        );
      }
    }
  }
}
