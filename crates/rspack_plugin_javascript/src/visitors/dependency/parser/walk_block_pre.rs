use swc_core::ecma::ast::{ClassDecl, ImportDecl, ImportSpecifier};
use swc_core::ecma::ast::{Decl, DefaultDecl, ExportAll, ExportDefaultDecl, ExprStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, NamedExport, Stmt, VarDecl, VarDeclKind};

use super::JavascriptParser;
use crate::parser_plugin::JavascriptParserPlugin;

impl<'parser> JavascriptParser<'parser> {
  pub fn block_pre_walk_module_declarations(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.block_pre_walk_module_declaration(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.block_pre_walk_statement(statement);
    }
  }

  fn block_pre_walk_module_declaration(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        // TODO: `hooks.block_pre_statement.call`
        match decl {
          ModuleDecl::Import(decl) => self.block_pre_walk_import_declaration(decl),
          ModuleDecl::ExportAll(decl) => self.block_pre_walk_export_all_declaration(decl),
          ModuleDecl::ExportDefaultDecl(decl) => {
            self.block_pre_walk_export_default_declaration(decl)
          }
          ModuleDecl::ExportNamed(decl) => self.block_pre_walk_export_name_declaration(decl),
          ModuleDecl::ExportDefaultExpr(_) => (),
          ModuleDecl::ExportDecl(_) => (),
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }
      ModuleItem::Stmt(stmt) => self.block_pre_walk_statement(stmt),
    }
  }

  fn block_pre_walk_statement(&mut self, stmt: &Stmt) {
    // TODO: `hooks.block_pre_statement.call`
    match stmt {
      Stmt::Decl(stmt) => match stmt {
        Decl::Class(decl) => self.block_pre_walk_class_declaration(decl),
        Decl::Var(decl) => self.block_pre_walk_variable_declaration(decl),
        Decl::Fn(_) | Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::Expr(expr) => self.block_pre_walk_expression_statement(expr),
      _ => (),
    }
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

  fn block_pre_walk_export_name_declaration(&mut self, decl: &NamedExport) {
    if let Some(source) = &decl.src {
      self
        .plugin_drive
        .clone()
        .named_export_import(self, decl, source.value.as_str());
    } else {
      // TODO: `hooks.export.call`
    }
  }

  fn block_pre_walk_class_declaration(&mut self, decl: &ClassDecl) {
    self.define_variable(decl.ident.sym.to_string())
  }

  fn block_pre_walk_export_default_declaration(&mut self, decl: &ExportDefaultDecl) {
    // FIXME: webpack use `self.pre_walk_statement(decl.decl)`
    match &decl.decl {
      DefaultDecl::Class(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.to_string())
        }
      }
      DefaultDecl::Fn(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.to_string())
        }
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }

    // FIXME: webpack use `self.block_pre_walk_statement(decl.decl)`
    // match &decl.decl {
    //   DefaultDecl::Class(expr) => {
    //     if let Some(ident) = &expr.ident {
    //       self.define_variable(ident.sym.to_string())
    //     }
    //   }
    //   DefaultDecl::Fn(expr) => {
    //     if let Some(ident) = &expr.ident {
    //       self.define_variable(ident.sym.to_string())
    //     }
    //   }
    //   DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    // }
  }

  fn block_pre_walk_export_all_declaration(&mut self, decl: &ExportAll) {
    self
      .plugin_drive
      .clone()
      .all_export_import(self, decl, decl.src.value.as_str());
    // TODO: `hooks.export_import_specifier.call`
  }

  fn block_pre_walk_import_declaration(&mut self, decl: &ImportDecl) {
    self
      .plugin_drive
      .clone()
      .import(self, decl, decl.src.value.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(named.local.sym.to_string())
        }
        ImportSpecifier::Default(default) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(default.local.sym.to_string())
        }
        ImportSpecifier::Namespace(namespace) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(namespace.local.sym.to_string());
        }
      }
    }
  }
}
