use swc_core::common::Spanned;
use swc_core::ecma::ast::{Decl, DefaultDecl, ExprStmt};
use swc_core::ecma::ast::{ExportSpecifier, ImportDecl, ImportSpecifier, ModuleExportName};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, Stmt, VarDecl, VarDeclKind};

use super::estree::{
  ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultExpression, ExportImport,
  ExportLocal, ExportNamedDeclaration, MaybeNamedClassDecl, MaybeNamedFunctionDecl, Statement,
};
use super::JavascriptParser;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::JS_DEFAULT_KEYWORD;

impl<'parser> JavascriptParser<'parser> {
  pub fn block_pre_walk_module_items(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.block_pre_walk_module_item(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.block_pre_walk_statement(statement.into());
    }
  }

  pub fn block_pre_walk_module_item(&mut self, statement: &ModuleItem) {
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
              ModuleDecl::Import(decl) => parser.block_pre_walk_import_declaration(decl),
              ModuleDecl::ExportAll(decl) => {
                parser.block_pre_walk_export_all_declaration(ExportAllDeclaration::All(decl))
              }
              ModuleDecl::ExportNamed(decl) => {
                if decl.specifiers.len() == 1
                  && matches!(decl.specifiers.first(), Some(ExportSpecifier::Namespace(_)))
                {
                  parser.block_pre_walk_export_all_declaration(ExportAllDeclaration::NamedAll(decl))
                } else {
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
              ModuleDecl::TsImportEquals(_)
              | ModuleDecl::TsExportAssignment(_)
              | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
            };
          },
        );
      }
      ModuleItem::Stmt(stmt) => self.block_pre_walk_statement(stmt.into()),
    }
  }

  pub fn block_pre_walk_statement(&mut self, stmt: Statement) {
    self.enter_statement(
      &stmt,
      |parser, _| {
        parser
          .plugin_drive
          .clone()
          .block_pre_statement(parser, stmt.into())
          .unwrap_or_default()
      },
      |parser, _| match stmt {
        Statement::Class(decl) => parser.block_pre_walk_class_declaration(decl.into()),
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

  pub(super) fn block_pre_walk_variable_declaration(&mut self, decl: &VarDecl) {
    if decl.kind != VarDeclKind::Var {
      self._pre_walk_variable_declaration(decl);
    }
  }

  fn block_pre_walk_class_declaration(&mut self, decl: MaybeNamedClassDecl) {
    if let Some(ident) = decl.ident {
      self.define_variable(ident.sym.to_string())
    }
  }

  fn block_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    if let Some(source) = export.source() {
      self
        .plugin_drive
        .clone()
        .export_import(self, ExportImport::Named(export), source);
    } else {
      self
        .plugin_drive
        .clone()
        .export(self, ExportLocal::Named(export));
    }
    match export {
      ExportNamedDeclaration::Decl(decl) => {
        let prev = self.prev_statement;
        self.pre_walk_statement((&decl.decl).into());
        self.prev_statement = prev;
        self.block_pre_walk_statement((&decl.decl).into());
        self.enter_declaration(&decl.decl, |parser, def| {
          parser.plugin_drive.clone().export_specifier(
            parser,
            ExportLocal::Named(export),
            &def.sym,
            &def.sym,
          );
        });
      }
      ExportNamedDeclaration::Specifiers(named) => {
        for spec in named.specifiers.iter() {
          let (local_id, exported_name) = match spec {
              ExportSpecifier::Namespace(_) => unreachable!("should handle ExportSpecifier::Namespace by ExportAllOrNamedAll::NamedAll in block_pre_walk_export_all_declaration"),
              ExportSpecifier::Default(s) => {
                (JS_DEFAULT_KEYWORD.clone(), s.exported.sym.clone())
              },
              ExportSpecifier::Named(n) => {
                (n.orig.atom().clone(), n.exported.as_ref().unwrap_or(&n.orig).atom().clone())
              },
            };
          if let Some(src) = &named.src {
            self.plugin_drive.clone().export_import_specifier(
              self,
              ExportImport::Named(export),
              &src.value,
              Some(&local_id),
              Some(&exported_name),
            );
          } else {
            self.plugin_drive.clone().export_specifier(
              self,
              ExportLocal::Named(export),
              &local_id,
              &exported_name,
            );
          }
        }
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
        match &decl.decl {
          DefaultDecl::Class(c) => {
            let stmt = Statement::Class(c.into());
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = &c.ident {
              self.plugin_drive.clone().export_specifier(
                self,
                ExportLocal::Default(export),
                &ident.sym,
                &JS_DEFAULT_KEYWORD,
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
            let stmt = Statement::Fn(f.into());
            let prev = self.prev_statement;
            self.pre_walk_statement(stmt);
            self.prev_statement = prev;
            self.block_pre_walk_statement(stmt);
            if let Some(ident) = &f.ident {
              self.plugin_drive.clone().export_specifier(
                self,
                ExportLocal::Default(export),
                &ident.sym,
                &JS_DEFAULT_KEYWORD,
              );
            } else {
              self.plugin_drive.clone().export_expression(
                self,
                export,
                ExportDefaultExpression::FnDecl(f),
              );
            }
          }
          DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
        };
      }
      ExportDefaultDeclaration::Expr(expr) => {
        // Webpack call exportExpression in walk (legacy code maybe)
        // We move it to block_pre_walk for consistent with other export related hook
        self.plugin_drive.clone().export_expression(
          self,
          export,
          ExportDefaultExpression::Expr(&expr.expr),
        );
      }
    }
  }

  fn block_pre_walk_export_all_declaration(&mut self, decl: ExportAllDeclaration) {
    let exported_name = decl.exported_name();
    let decl = ExportImport::All(decl);
    let source = decl.source();
    self.plugin_drive.clone().export_import(self, decl, source);
    self
      .plugin_drive
      .clone()
      .export_import_specifier(self, decl, source, None, exported_name);
  }

  fn block_pre_walk_import_declaration(&mut self, decl: &ImportDecl) {
    let drive = self.plugin_drive.clone();
    let source = &decl.src.value;
    drive.import(self, decl, source.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          let identifier_name = &named.local.sym;
          let export_name = named
            .imported
            .as_ref()
            .map(|imported| match imported {
              ModuleExportName::Ident(ident) => &ident.sym,
              ModuleExportName::Str(s) => &s.value,
            })
            .unwrap_or_else(|| &named.local.sym);
          if drive
            .import_specifier(self, decl, source, Some(export_name), identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Default(default) => {
          let identifier_name = &default.local.sym;
          if drive
            .import_specifier(self, decl, source, Some(&"default".into()), identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Namespace(namespace) => {
          let identifier_name = &namespace.local.sym;
          if drive
            .import_specifier(self, decl, source, None, identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
      }
    }
  }
}
