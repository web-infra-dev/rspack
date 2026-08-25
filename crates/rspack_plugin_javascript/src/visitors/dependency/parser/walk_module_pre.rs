use swc_atoms::Atom;
use swc_next_ecma_ast::{
  GetSpan, ImportDeclaration, ImportDeclarationSpecifierData, Stmt, StmtData, TypedSubRange,
};

use crate::{
  JavascriptParserPlugin,
  visitors::{
    ExportAllDeclaration, ExportImport, ExportNamedDeclaration, JavascriptParser,
    module_export_name_to_atom,
  },
};

impl JavascriptParser<'_> {
  pub fn module_pre_walk_module_items(&mut self, statements: TypedSubRange<Stmt>) {
    let ast = self.ast.ast;
    for id in statements.iter() {
      let statement = ast.get_node_in_sub_range(id);
      self.statement_path.push(statement.span(ast).into());
      match ast.stmt_data(statement) {
        StmtData::ImportDeclaration(declaration) => {
          self.module_pre_walk_import_declaration(declaration);
        }
        StmtData::ExportAllDeclaration(declaration) => {
          self.module_pre_walk_export_all_declaration(ExportAllDeclaration(declaration));
        }
        StmtData::ExportNamedDeclaration(declaration) => {
          self.module_pre_walk_export_named_declaration(ExportNamedDeclaration(declaration));
        }
        _ => {}
      }
      self.prev_statement = self.statement_path.pop();
    }
  }

  pub fn module_pre_walk_import_declaration(&mut self, declaration: ImportDeclaration) {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    let source = ast
      .get_wtf8(declaration.source(ast).value(ast))
      .to_string_lossy()
      .into_owned();
    drive.import(self, declaration, &source);
    let source_atom = Atom::from(source);
    for id in declaration.specifiers(ast).iter() {
      let specifier = ast.get_node_in_sub_range(id);
      match ast.import_declaration_specifier_data(specifier) {
        ImportDeclarationSpecifierData::ImportSpecifier(named) => {
          let local = named.local(ast);
          let identifier_name = Atom::from(ast.get_utf8(local.name(ast)));
          let export_name = module_export_name_to_atom(ast, named.imported(ast));
          if drive
            .import_specifier(
              self,
              declaration,
              &source_atom,
              Some(&export_name),
              &identifier_name,
            )
            .unwrap_or_default()
          {
            self.define_variable(identifier_name);
          }
        }
        ImportDeclarationSpecifierData::ImportDefaultSpecifier(default) => {
          let identifier_name = Atom::from(ast.get_utf8(default.local(ast).name(ast)));
          if drive
            .import_specifier(
              self,
              declaration,
              &source_atom,
              Some(&"default".into()),
              &identifier_name,
            )
            .unwrap_or_default()
          {
            self.define_variable(identifier_name);
          }
        }
        ImportDeclarationSpecifierData::ImportNamespaceSpecifier(namespace) => {
          let identifier_name = Atom::from(ast.get_utf8(namespace.local(ast).name(ast)));
          if drive
            .import_specifier(self, declaration, &source_atom, None, &identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name);
          }
        }
      }
    }
  }

  pub fn module_pre_walk_export_all_declaration(&mut self, declaration: ExportAllDeclaration) {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    let exported_name = declaration.exported_name(ast);
    let exported_name_span = declaration.exported_name_span(ast);
    let statement = ExportImport::All(declaration);
    let source = statement.source(ast);
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
    let ast = self.ast.ast;
    let Some(source) = export.source(ast) else {
      return;
    };
    let drive = self.plugin_drive.clone();
    drive.export_import(self, ExportImport::Named(export), &source);
    for (local, exported, span) in export.named_export_specifiers(ast) {
      drive.export_import_specifier(
        self,
        ExportImport::Named(export),
        &source,
        Some(&local),
        Some(&exported),
        Some(span),
      );
    }
  }
}
