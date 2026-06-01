use swc_core::{
  common::Spanned,
  ecma::ast::{
    ExportSpecifier, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
  },
};

use crate::{
  JavascriptParserPlugin,
  visitors::{ExportAllDeclaration, ExportImport, ExportNamedDeclaration, JavascriptParser},
};

impl JavascriptParser<'_> {
  pub fn module_pre_walk_module_items(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.statement_path.push(statement.span().into());
      match statement {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
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

  pub fn module_pre_walk_import_declaration(&mut self, decl: &ImportDecl) {
    let drive = self.plugin_drive.clone();
    let atom = decl.src.value.to_atom_lossy();
    let source = atom.as_ref();
    drive.import(self, decl, source.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          let local = &named.local.to_id();
          let export_name = named
            .imported
            .as_ref()
            .map_or(&named.local.sym, |imported| match imported {
              ModuleExportName::Ident(ident) => &ident.sym,
              ModuleExportName::Str(s) => s
                .value
                .as_atom()
                .expect("ModuleExportName should be a valid utf8"),
            });
          drive.import_specifier(self, decl, source, Some(export_name), local);
        }
        ImportSpecifier::Default(default) => {
          let local = &default.local.to_id();
          drive.import_specifier(self, decl, source, Some(&"default".into()), local);
        }
        ImportSpecifier::Namespace(namespace) => {
          let local = &namespace.local.to_id();
          drive.import_specifier(self, decl, source, None, local);
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
    drive.export_import(self, statement, source);
    drive.export_import_specifier(
      self,
      statement,
      source,
      None,
      exported_name,
      exported_name_span,
    );
  }

  pub fn module_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    let Some(source) = export.source() else {
      return;
    };
    let drive = self.plugin_drive.clone();
    drive.export_import(self, ExportImport::Named(export), source);
    match export {
      ExportNamedDeclaration::Decl(_) => {}
      ExportNamedDeclaration::Specifiers(named) => {
        for (local_id, exported_name, exported_name_span) in
          ExportNamedDeclaration::named_export_specifiers(named)
        {
          drive.export_import_specifier(
            self,
            ExportImport::Named(export),
            source,
            Some(&local_id),
            Some(&exported_name),
            Some(exported_name_span),
          );
        }
      }
    }
  }
}
