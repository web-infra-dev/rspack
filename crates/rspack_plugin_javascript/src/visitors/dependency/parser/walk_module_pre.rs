use swc_experimental_ecma_ast::{
  ExportSpecifier, GetSpan, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
  TypedSubRange,
};

use crate::{
  JavascriptParserPlugin,
  visitors::{ExportAllDeclaration, ExportImport, ExportNamedDeclaration, JavascriptParser},
};

impl JavascriptParser<'_> {
  pub fn module_pre_walk_module_items(&mut self, statements: TypedSubRange<ModuleItem>) {
    for statement in statements.iter() {
      let statement = self.ast.get_node_in_sub_range(statement);
      self.statement_path.push(statement.span(&self.ast).into());
      match statement {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          ModuleDecl::Import(decl) => self.module_pre_walk_import_declaration(decl),
          ModuleDecl::ExportAll(decl) => {
            self.module_pre_walk_export_all_declaration(ExportAllDeclaration::All(decl))
          }
          ModuleDecl::ExportNamed(decl) => {
            let is_named_namespace_export = decl.specifiers(&self.ast).len() == 1
              && matches!(
                decl
                  .specifiers(&self.ast)
                  .first()
                  .map(|n| self.ast.get_node_in_sub_range(n)),
                Some(ExportSpecifier::Namespace(_))
              );
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

  pub fn module_pre_walk_import_declaration(&mut self, decl: ImportDecl) {
    let drive = self.plugin_drive.clone();
    let atom = self
      .ast
      .get_wtf8_atom(decl.src(&self.ast).value(&self.ast))
      .to_atom_lossy()
      .into_owned();
    drive.import(self, decl, atom.as_str());

    for specifier in decl.specifiers(&self.ast).iter() {
      let specifier = self.ast.get_node_in_sub_range(specifier);
      match specifier {
        ImportSpecifier::Named(named) => {
          let identifier_name = self.ast.get_atom(named.local(&self.ast).sym(&self.ast));
          let export_name = named.imported(&self.ast).map_or(
            self.ast.get_atom(named.local(&self.ast).sym(&self.ast)),
            |imported| match imported {
              ModuleExportName::Ident(ident) => self.ast.get_atom(ident.sym(&self.ast)),
              ModuleExportName::Str(s) => self
                .ast
                .get_wtf8_atom(s.value(&self.ast))
                .to_atom_lossy()
                .into_owned(),
            },
          );
          if drive
            .import_specifier(self, decl, &atom, Some(&export_name), &identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name)
          }
        }
        ImportSpecifier::Default(default) => {
          let identifier_name = self.ast.get_atom(default.local(&self.ast).sym(&self.ast));
          if drive
            .import_specifier(self, decl, &atom, Some(&"default".into()), &identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.clone())
          }
        }
        ImportSpecifier::Namespace(namespace) => {
          let identifier_name = self.ast.get_atom(namespace.local(&self.ast).sym(&self.ast));
          if drive
            .import_specifier(self, decl, &atom, None, &identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name)
          }
        }
      }
    }
  }

  pub fn module_pre_walk_export_all_declaration(&mut self, decl: ExportAllDeclaration) {
    let exported_name = decl.exported_name(&self.ast);
    let exported_name_span = decl.exported_name_span(&self.ast);
    let statement = ExportImport::All(decl);
    let source = statement.source(&self.ast);
    self
      .plugin_drive
      .clone()
      .export_import(self, statement, &source);
    self.plugin_drive.clone().export_import_specifier(
      self,
      statement,
      &source,
      None,
      exported_name.as_ref(),
      exported_name_span,
    );
  }

  pub fn module_pre_walk_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
    let Some(source) = export.source(&self.ast) else {
      return;
    };
    self
      .plugin_drive
      .clone()
      .export_import(self, ExportImport::Named(export), &source);
    match export {
      ExportNamedDeclaration::Decl(_) => {}
      ExportNamedDeclaration::Specifiers(named) => {
        ExportNamedDeclaration::named_export_specifiers(
          self,
          named,
          |this, local_id, exported_name, exported_name_span| {
            this.plugin_drive.clone().export_import_specifier(
              this,
              ExportImport::Named(export),
              &source,
              Some(&local_id),
              Some(&exported_name),
              Some(exported_name_span),
            );
          },
        );
      }
    }
  }
}
