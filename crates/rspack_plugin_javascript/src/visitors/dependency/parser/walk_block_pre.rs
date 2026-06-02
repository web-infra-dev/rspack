use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  DefaultDecl, ExportSpecifier, ExprStmt, ModuleDecl, ModuleItem, Stmt,
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
  visitors::{VariableDeclaration, VariableDeclarationKind},
};

impl JavascriptParser<'_> {
  pub fn block_pre_walk_module_items(&mut self, statements: &[ModuleItem<'_>]) {
    for statement in statements {
      self.block_pre_walk_module_item(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: &[Stmt<'_>]) {
    for statement in statements {
      self.block_pre_walk_statement(statement.into());
    }
  }

  pub fn block_pre_walk_module_item(&mut self, statement: &ModuleItem<'_>) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        let drive = self.plugin_drive.clone();
        self.enter_statement(
          &**decl,
          |parser, _| {
            drive
              .block_pre_module_declaration(parser, decl)
              .unwrap_or_default()
          },
          |parser, _| {
            match &**decl {
              ModuleDecl::Import(_) => {}
              ModuleDecl::ExportAll(_) => {}
              ModuleDecl::ExportNamed(decl) => {
                let is_named_namespace_export = decl.specifiers.len() == 1
                  && matches!(decl.specifiers.first(), Some(ExportSpecifier::Namespace(_)));
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
      ModuleItem::Stmt(stmt) => self.block_pre_walk_statement((&**stmt).into()),
    }
  }

  pub fn block_pre_walk_statement(&mut self, stmt: Statement) {
    let drive = self.plugin_drive.clone();
    self.enter_statement(
      &stmt,
      |parser, _| drive.block_pre_statement(parser, stmt).unwrap_or_default(),
      |parser, _| match stmt {
        Statement::Class(decl) => parser.block_pre_walk_class_declaration(decl),
        Statement::Var(decl) => parser.block_pre_walk_variable_declaration(decl),
        Statement::Expr(expr) => parser.block_pre_walk_expression_statement(expr),
        _ => (),
      },
    );
  }

  fn block_pre_walk_expression_statement(&mut self, stmt: &ExprStmt) {
    if let Some(assign) = stmt.expr.as_assign() {
      self.pre_walk_assignment_expression(assign)
    }
  }

  pub(super) fn block_pre_walk_variable_declaration(&mut self, decl: VariableDeclaration<'_>) {
    if decl.kind() != VariableDeclarationKind::Var {
      self._pre_walk_variable_declaration(decl);
    }
  }

  fn block_pre_walk_class_declaration(&mut self, decl: MaybeNamedClassDecl) {
    if let Some(ident) = decl.ident() {
      self.define_variable(Atom::from(ident.sym.as_str()))
    }
  }

  fn block_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    if export.source().is_some() {
      return;
    }
    let drive = self.plugin_drive.clone();
    drive.export(self, ExportLocal::Named(export));
    match export {
      ExportNamedDeclaration::Decl(decl) => {
        let prev = self.prev_statement;
        self.pre_walk_statement((&decl.decl).into());
        self.prev_statement = prev;
        self.block_pre_walk_statement((&decl.decl).into());
        self.enter_declaration(&decl.decl, |parser, def| {
          drive.export_specifier(
            parser,
            ExportLocal::Named(export),
            &Atom::from(def.sym.as_str()),
            &Atom::from(def.sym.as_str()),
            def.span,
          );
        });
      }
      ExportNamedDeclaration::Specifiers(named) => {
        for (local_id, exported_name, exported_name_span) in
          ExportNamedDeclaration::named_export_specifiers(named)
        {
          if named.src.is_none() {
            drive.export_specifier(
              self,
              ExportLocal::Named(export),
              &local_id,
              &exported_name,
              exported_name_span,
            );
          }
        }
      }
    }
  }

  fn block_pre_walk_export_default_declaration(&mut self, export: ExportDefaultDeclaration) {
    let drive = self.plugin_drive.clone();
    drive.export(self, ExportLocal::Default(export));
    match export {
      ExportDefaultDeclaration::Decl(decl) => {
        match &decl.decl {
          DefaultDecl::Class(c) => {
            let stmt = Statement::Class((&**c).into());
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = c.ident.as_deref() {
              drive.export_specifier(
                self,
                ExportLocal::Default(export),
                &Atom::from(ident.sym.as_str()),
                &JS_DEFAULT_KEYWORD,
                ident.span,
              );
            } else {
              drive.export_expression(self, export, ExportDefaultExpression::ClassDecl(c));
            }
          }
          DefaultDecl::Fn(f) => {
            let stmt = Statement::Fn((&**f).into());
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = f.ident.as_deref() {
              drive.export_specifier(
                self,
                ExportLocal::Default(export),
                &Atom::from(ident.sym.as_str()),
                &JS_DEFAULT_KEYWORD,
                ident.span,
              );
            } else {
              drive.export_expression(self, export, ExportDefaultExpression::FnDecl(f));
            }
          }
        };
      }
      ExportDefaultDeclaration::Expr(expr) => {
        // Webpack call exportExpression in walk (legacy code maybe)
        // We move it to block_pre_walk for consistent with other export related hook
        drive.export_expression(self, export, ExportDefaultExpression::Expr(&expr.expr));
      }
    }
  }
}
