use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  ExportSpecifier, GetSpan, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
};

use crate::{
  JavascriptParserPlugin,
  visitors::{ExportAllDeclaration, ExportImport, ExportNamedDeclaration, JavascriptParser},
};

impl JavascriptParser<'_> {
  pub fn module_pre_walk_module_items(&mut self, statements: &[ModuleItem<'_>]) {
    for statement in statements {
      self.statement_path.push(statement.span().into());
      match statement {
        ModuleItem::ModuleDecl(module_decl) => match &**module_decl {
          ModuleDecl::Import(decl) => self.module_pre_walk_import_declaration(decl),
          ModuleDecl::ExportAll(decl) => {
            self.module_pre_walk_export_all_declaration(ExportAllDeclaration::All(decl))
          }
          ModuleDecl::ExportNamed(decl) => {
            let is_named_namespace_export = decl.specifiers.len() == 1
              && matches!(decl.specifiers.first(), Some(ExportSpecifier::Namespace(_)));
            if is_named_namespace_export {
              self.module_pre_walk_export_all_declaration(ExportAllDeclaration::NamedAll(decl))
            } else {
              self
                .module_pre_walk_export_named_declaration(ExportNamedDeclaration::Specifiers(decl))
            }
          }
          _ => {}
        },
        ModuleItem::Stmt(_) => {}
      }
      self.prev_statement = self.statement_path.pop();
    }
  }

  pub fn module_pre_walk_import_declaration(&mut self, decl: &ImportDecl<'_>) {
    let drive = self.plugin_drive.clone();
    let source = decl.src.value.as_wtf8().to_string_lossy().to_string();
    drive.import(self, decl, source.as_str());
    let source_atom = Atom::from(source.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          let identifier_name = Atom::from(named.local.sym.as_str());
          let export_name = named.imported.as_ref().map_or_else(
            || identifier_name.clone(),
            |imported| match imported {
              ModuleExportName::Ident(ident) => Atom::from(ident.sym.as_str()),
              ModuleExportName::Str(s) => Atom::from(s.value.as_wtf8().to_string_lossy().as_ref()),
            },
          );
          if drive
            .import_specifier(
              self,
              decl,
              &source_atom,
              Some(&export_name),
              &identifier_name,
            )
            .unwrap_or_default()
          {
            self.define_variable(identifier_name)
          }
        }
        ImportSpecifier::Default(default) => {
          let identifier_name = Atom::from(default.local.sym.as_str());
          if drive
            .import_specifier(
              self,
              decl,
              &source_atom,
              Some(&"default".into()),
              &identifier_name,
            )
            .unwrap_or_default()
          {
            self.define_variable(identifier_name)
          }
        }
        ImportSpecifier::Namespace(namespace) => {
          let identifier_name = Atom::from(namespace.local.sym.as_str());
          if drive
            .import_specifier(self, decl, &source_atom, None, &identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name)
          }
        }
      }
    }
  }

  pub fn module_pre_walk_export_all_declaration(&mut self, decl: ExportAllDeclaration) {
    let drive = self.plugin_drive.clone();
    let exported_name = decl.exported_name();
    let exported_name_span = decl.exported_name_span();
    let statement = ExportImport::All(decl);
    let source = statement.source();
    drive.export_import(self, statement, &source);
    drive.export_import_specifier(
      self,
      statement,
      &source,
      None,
      exported_name.as_ref(),
      exported_name_span,
    );
  }

  pub fn module_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    let Some(source) = export.source() else {
      return;
    };
    let drive = self.plugin_drive.clone();
    drive.export_import(self, ExportImport::Named(export), &source);
    match export {
      ExportNamedDeclaration::Decl(_) => {}
      ExportNamedDeclaration::Specifiers(named) => {
        for (local_id, exported_name, exported_name_span) in
          ExportNamedDeclaration::named_export_specifiers(named)
        {
          drive.export_import_specifier(
            self,
            ExportImport::Named(export),
            &source,
            Some(&local_id),
            Some(&exported_name),
            Some(exported_name_span),
          );
        }
      }
    }
  }
}
