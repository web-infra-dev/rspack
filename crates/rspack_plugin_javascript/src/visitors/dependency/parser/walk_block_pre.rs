use swc_core::ecma::ast::{ClassDecl, ImportDecl, ImportSpecifier, ModuleExportName};
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
        if self
          .plugin_drive
          .clone()
          .block_pre_module_declration(self, decl)
          .unwrap_or_default()
        {
          return;
        }

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
    if self
      .plugin_drive
      .clone()
      .block_pre_statement(self, stmt)
      .unwrap_or_default()
    {
      return;
    }

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
    // Due to type incompatibility, it does not function in the same way as webpack.
    self
      .plugin_drive
      .clone()
      .block_pre_walk_export_default_declaration(self, decl)
      .unwrap_or_default();

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
  }

  fn block_pre_walk_export_all_declaration(&mut self, decl: &ExportAll) {
    self
      .plugin_drive
      .clone()
      .all_export_import(self, decl, decl.src.value.as_str());
    // TODO: `hooks.export_import_specifier.call`
  }

  fn block_pre_walk_import_declaration(&mut self, decl: &ImportDecl) {
    let drive = self.plugin_drive.clone();
    let source = &decl.src.value;
    drive.import(self, decl, source.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          let identifier_name = named.local.sym.as_str();
          let export_name = named.imported.as_ref().map(|imported| match imported {
            ModuleExportName::Ident(ident) => ident.sym.as_str(),
            ModuleExportName::Str(s) => s.value.as_str(),
          });
          if drive
            .import_specifier(self, decl, source, export_name, identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Default(default) => {
          let identifier_name = default.local.sym.as_str();
          if drive
            .import_specifier(self, decl, source, Some("default"), identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Namespace(namespace) => {
          let identifier_name = namespace.local.sym.as_str();
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
