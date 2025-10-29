use rspack_util::atom::ModuleExportNameExt;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  atoms::Wtf8Atom,
  ecma::{
    ast::{Decl, ExportSpecifier, ModuleDecl, ModuleItem, Program, Stmt},
    visit::Visit,
  },
};

#[derive(Debug)]
pub struct TypeExportsCollector<'a> {
  type_idents: FxHashSet<Wtf8Atom>,
  export_idents: FxHashMap<Wtf8Atom, Wtf8Atom>,

  type_exports: &'a mut FxHashSet<Wtf8Atom>,
}

impl<'a> TypeExportsCollector<'a> {
  pub fn new(type_exports: &'a mut FxHashSet<Wtf8Atom>) -> Self {
    Self {
      type_idents: Default::default(),
      export_idents: Default::default(),
      type_exports,
    }
  }
}

impl Visit for TypeExportsCollector<'_> {
  fn visit_program(&mut self, node: &Program) {
    let Program::Module(node) = node else {
      return;
    };
    for item in &node.body {
      match item {
        ModuleItem::ModuleDecl(decl) => match decl {
          ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
            Decl::TsInterface(interface_decl) => {
              self
                .type_idents
                .insert(interface_decl.id.sym.clone().into());
              self
                .type_exports
                .insert(interface_decl.id.sym.clone().into());
            }
            Decl::TsTypeAlias(type_alias_decl) => {
              self
                .type_idents
                .insert(type_alias_decl.id.sym.clone().into());
              self
                .type_exports
                .insert(type_alias_decl.id.sym.clone().into());
            }
            Decl::TsEnum(enum_decl) => {
              self.type_idents.insert(enum_decl.id.sym.clone().into());
              self.type_exports.insert(enum_decl.id.sym.clone().into());
            }
            _ => {}
          },
          ModuleDecl::ExportNamed(named_export) => {
            if named_export.type_only {
              self
                .type_exports
                .extend(named_export.specifiers.iter().filter_map(|specifier| {
                  match specifier {
                    ExportSpecifier::Named(specifier) => Some(
                      specifier
                        .exported
                        .as_ref()
                        .unwrap_or(&specifier.orig)
                        .wtf8()
                        .clone(),
                    ),
                    _ => None,
                  }
                }));
            } else {
              for specifier in &named_export.specifiers {
                match specifier {
                  ExportSpecifier::Named(specifier) => {
                    if specifier.is_type_only {
                      self.type_exports.insert(
                        specifier
                          .exported
                          .as_ref()
                          .unwrap_or(&specifier.orig)
                          .wtf8()
                          .clone(),
                      );
                    } else if named_export.src.is_none() {
                      self.export_idents.insert(
                        specifier.orig.wtf8().clone(),
                        specifier
                          .exported
                          .as_ref()
                          .unwrap_or(&specifier.orig)
                          .wtf8()
                          .clone(),
                      );
                    }
                  }
                  _ => continue,
                }
              }
            }
          }
          ModuleDecl::ExportDefaultDecl(decl) if decl.decl.is_ts_interface_decl() => {
            self.type_exports.insert("default".into());
          }
          ModuleDecl::ExportDefaultExpr(expr) => {
            if let Some(ident) = expr.expr.unwrap_parens().as_ident() {
              self
                .export_idents
                .insert(ident.sym.clone().into(), "default".into());
            }
          }
          _ => {}
        },
        ModuleItem::Stmt(stmt) => {
          if let Stmt::Decl(decl) = stmt {
            match decl {
              Decl::TsInterface(interface_decl) => {
                self
                  .type_idents
                  .insert(interface_decl.id.sym.clone().into());
              }
              Decl::TsTypeAlias(type_alias_decl) => {
                self
                  .type_idents
                  .insert(type_alias_decl.id.sym.clone().into());
              }
              Decl::TsEnum(enum_decl) => {
                self.type_idents.insert(enum_decl.id.sym.clone().into());
              }
              _ => {}
            }
          }
        }
      }
    }

    self.type_exports.extend(
      self
        .export_idents
        .iter()
        .filter(|(export_ident, _)| self.type_idents.contains(export_ident))
        .map(|(_, exported_as)| exported_as.clone()),
    );
  }
}
