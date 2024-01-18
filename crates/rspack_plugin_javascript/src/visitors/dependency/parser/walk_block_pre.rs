use swc_core::ecma::ast::{ClassDecl, ImportDecl, ImportSpecifier};
use swc_core::ecma::ast::{Decl, DefaultDecl, ExportAll, ExportDefaultDecl, ExprStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, NamedExport, Stmt, VarDecl, VarDeclKind};

use super::JavascriptParser;
use crate::parser_plugin::JavaScriptParserPluginDrive;

impl<'ast, 'parser> JavascriptParser<'parser> {
  pub fn block_pre_walk_module_declarations(
    &mut self,
    statements: &'ast Vec<ModuleItem>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.block_pre_walk_module_declaration(statement, plugin_drive);
    }
  }

  pub fn block_pre_walk_statements(
    &mut self,
    statements: &'ast Vec<Stmt>,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    for statement in statements {
      self.block_pre_walk_statement(statement, plugin_drive);
    }
  }

  fn block_pre_walk_module_declaration(
    &mut self,
    statement: &ModuleItem,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        // TODO: `hooks.block_pre_statement.call`
        match decl {
          ModuleDecl::Import(decl) => self.block_pre_walk_import_declaration(decl, plugin_drive),
          ModuleDecl::ExportAll(decl) => {
            self.block_pre_walk_export_all_declaration(decl, plugin_drive)
          }
          ModuleDecl::ExportDefaultDecl(decl) => {
            self.block_pre_walk_export_default_declaration(decl, plugin_drive)
          }
          ModuleDecl::ExportNamed(decl) => {
            self.block_pre_walk_export_name_declaration(decl, plugin_drive)
          }
          ModuleDecl::ExportDefaultExpr(_) => (),
          ModuleDecl::ExportDecl(_) => (),
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }
      ModuleItem::Stmt(stmt) => self.block_pre_walk_statement(stmt, plugin_drive),
    }
  }

  fn block_pre_walk_statement(
    &mut self,
    stmt: &Stmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `hooks.block_pre_statement.call`
    match stmt {
      Stmt::Decl(stmt) => match stmt {
        Decl::Class(decl) => self.block_pre_walk_class_declaration(decl, plugin_drive),
        Decl::Var(decl) => self.block_pre_walk_variable_declaration(decl, plugin_drive),
        Decl::Fn(_) | Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::Expr(expr) => self.block_pre_walk_expression_statement(expr, plugin_drive),
      _ => (),
    }
  }

  fn block_pre_walk_expression_statement(
    &mut self,
    stmt: &ExprStmt,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if let Some(assign) = stmt.expr.as_assign() {
      self.pre_walk_assignment_expression(assign, plugin_drive);
    }
  }

  pub(super) fn block_pre_walk_variable_declaration(
    &mut self,
    decl: &VarDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    if decl.kind != VarDeclKind::Var {
      self._pre_walk_variable_declaration(decl, plugin_drive);
    }
  }

  fn block_pre_walk_export_name_declaration(
    &mut self,
    _decl: &NamedExport,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // if let Some(source) = decl.src {
    //   // TODO: `hooks.export_import.call`
    // } else {
    //   // TODO: `hooks.export.call`
    // }
  }

  fn block_pre_walk_class_declaration(
    &mut self,
    decl: &ClassDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    self.define_variable(decl.ident.sym.as_str())
  }

  fn block_pre_walk_export_default_declaration(
    &mut self,
    decl: &ExportDefaultDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // FIXME: webpack use `self.pre_walk_statement(decl.decl)`
    match &decl.decl {
      DefaultDecl::Class(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.as_str())
        }
      }
      DefaultDecl::Fn(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.as_str())
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

  fn block_pre_walk_export_all_declaration(
    &mut self,
    _decl: &ExportAll,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `hooks.export_import.call`
    // TODO: `hooks.export_import_specifier.call`
  }

  fn block_pre_walk_import_declaration(
    &mut self,
    decl: &ImportDecl,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) {
    // TODO: `hooks.import.call`
    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(named.local.sym.as_str())
        }
        ImportSpecifier::Default(default) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(default.local.sym.as_str())
        }
        ImportSpecifier::Namespace(namespace) => {
          // TODO: `hooks.import_specifier.call`
          self.define_variable(namespace.local.sym.as_str());
        }
      }
    }
  }
}
