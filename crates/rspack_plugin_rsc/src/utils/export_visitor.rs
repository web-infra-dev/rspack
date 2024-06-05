// Based on https://github.com/fz6m/rs-module-lexer
use swc_core::ecma::ast::{self};
use swc_core::ecma::visit::{Visit, VisitWith};

pub static DEFAULT_EXPORT: &'static str = "default";

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
  #[doc = " Export name "]
  pub n: String,
  #[doc = " Export origin name "]
  pub ln: Option<String>,
}

pub struct ImportExportVisitor {
  pub exports: Vec<ExportSpecifier>,
}

impl ImportExportVisitor {
  pub fn new() -> Self {
    Self { exports: vec![] }
  }
}

// export
impl ImportExportVisitor {
  fn add_export(&mut self, export: ExportSpecifier) {
    self.exports.push(export);
  }

  fn add_export_from_ident(&mut self, ident: &ast::Ident) {
    let name = ident.sym.to_string();
    self.add_export(ExportSpecifier {
      n: name.clone(),
      ln: Some(name),
    })
  }

  fn parse_export_spec(&mut self, specifier: &ast::ExportSpecifier) -> bool {
    match specifier {
      ast::ExportSpecifier::Named(named) => {
        // skip type
        if named.is_type_only {
          return false;
        }

        let mut is_renamed = false;
        let name = if let Some(exported) = &named.exported {
          // export { a as b }
          is_renamed = true;
          match exported {
            ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
            // export { 'a' as 'b' }
            ast::ModuleExportName::Str(str) => str.value.to_string(),
          }
        } else {
          match &named.orig {
            // export { a }
            ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
            // export { "a" }
            ast::ModuleExportName::Str(str) => str.value.to_string(),
          }
        };

        let origin_name;
        if is_renamed {
          match &named.orig {
            ast::ModuleExportName::Ident(ident) => {
              origin_name = Some(ident.sym.to_string());
            }
            // export { 'a' as 'b' }
            ast::ModuleExportName::Str(str) => {
              origin_name = Some(str.value.to_string());
            }
          }
        } else {
          origin_name = Some(name.clone());
        }

        self.add_export(ExportSpecifier {
          n: name,
          ln: origin_name,
        });

        return true;
      }
      // export v from 'm'
      // current not support
      ast::ExportSpecifier::Default(_) => {
        return false;
      }
      // export * as a from 'b'
      ast::ExportSpecifier::Namespace(namespace) => {
        if let ast::ModuleExportName::Ident(ident) = &namespace.name {
          let name = ident.sym.to_string();
          self.add_export(ExportSpecifier { n: name, ln: None });
          return true;
        }
        return false;
      }
    }
  }

  fn parse_named_export(&mut self, export: &ast::NamedExport) -> bool {
    // export type { a } from 'b'
    // export type * as a from 'b'
    if export.type_only {
      return false;
    }

    // export { type c } from 'b'
    let is_all_type_export = export.specifiers.iter().all(|specifier| match specifier {
      ast::ExportSpecifier::Named(named) => named.is_type_only,
      _ => false,
    });
    if is_all_type_export {
      return false;
    }

    let mut is_need_add_import = false;
    for specifier in &export.specifiers {
      let need_add_import = self.parse_export_spec(specifier);
      if need_add_import && !is_need_add_import {
        is_need_add_import = true;
      }
    }
    return is_need_add_import;
  }

  fn parse_default_export_expr(&mut self, _: &ast::ExportDefaultExpr) {
    let name = DEFAULT_EXPORT.to_string();
    // find 'default' index start
    self.add_export(ExportSpecifier { n: name, ln: None })
  }

  fn parse_export_decl(&mut self, export: &ast::ExportDecl) -> bool {
    let mut need_eager_return = false;
    match &export.decl {
      ast::Decl::Class(decl) => self.add_export_from_ident(&decl.ident),
      ast::Decl::Fn(decl) => self.add_export_from_ident(&decl.ident),
      ast::Decl::Var(decl) => {
        decl.decls.iter().for_each(|decl| {
          // support export const a = 1, b = 2
          match &decl.name {
            ast::Pat::Ident(ident) => {
              let name = ident.sym.to_string();
              self.add_export(ExportSpecifier {
                n: name.clone(),
                ln: Some(name),
              })
            }
            ast::Pat::Object(pat) => {
              pat.props.iter().for_each(|prop| {
                match &prop {
                  // export const { a, b } = {}
                  ast::ObjectPatProp::Assign(assign) => {
                    let ident = &assign.key;
                    let name = ident.sym.to_string();
                    self.add_export(ExportSpecifier {
                      n: name.clone(),
                      ln: Some(name),
                    })
                  }
                  ast::ObjectPatProp::KeyValue(kv) => {
                    match kv.value.as_ref() {
                      ast::Pat::Ident(ident) => {
                        // only support value is ident
                        let name = ident.sym.to_string();
                        self.add_export(ExportSpecifier {
                          n: name.clone(),
                          ln: Some(name),
                        })
                      }
                      _ => {
                        // Not support
                      }
                    }
                  }
                  // Not support case: export const { a, ...b } = {}
                  // es-module-lexer not support find the `b` index
                  ast::ObjectPatProp::Rest(_) => {}
                }
              })
            }
            ast::Pat::Array(pat) => {
              pat.elems.iter().for_each(|elm| {
                if elm.is_some() {
                  // only support export const [a, b] = []
                  if let ast::Pat::Ident(ident) = &elm.as_ref().unwrap() {
                    let name = ident.sym.to_string();
                    self.add_export(ExportSpecifier {
                      n: name.clone(),
                      ln: Some(name),
                    })
                  }
                }
              })
            }
            _ => {}
          }
        })
      }
      ast::Decl::Using(_) => {}
      ast::Decl::TsEnum(decl) => {
        let name = decl.id.sym.to_string();
        self.add_export(ExportSpecifier {
          n: name.clone(),
          ln: Some(name),
        })
      }
      ast::Decl::TsModule(decl) => {
        if let ast::TsModuleName::Ident(ident) = &decl.id {
          let name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: name.clone(),
            ln: Some(name),
          })
        }
        // do not visit import / export within namespace
        need_eager_return = true;
      }
      ast::Decl::TsInterface(_) => {}
      ast::Decl::TsTypeAlias(_) => {}
    }
    need_eager_return
  }

  fn parse_export_default_decl(&mut self, export: &ast::ExportDefaultDecl) {
    match &export.decl {
      // export default class A {}
      // export default class {}
      ast::DefaultDecl::Class(decl) => {
        if let Some(ident) = &decl.ident {
          let origin_name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: DEFAULT_EXPORT.to_string(),
            ln: Some(origin_name),
          })
        } else {
          let name = DEFAULT_EXPORT.to_string();
          self.add_export(ExportSpecifier { n: name, ln: None })
        }
      }
      // export default function A() {}
      // export default function() {}
      ast::DefaultDecl::Fn(decl) => {
        if let Some(ident) = &decl.ident {
          let origin_name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: DEFAULT_EXPORT.to_string(),
            ln: Some(origin_name),
          })
        } else {
          let name = DEFAULT_EXPORT.to_string();
          self.add_export(ExportSpecifier {
            n: name.clone(),
            ln: None,
          })
        }
      }
      ast::DefaultDecl::TsInterfaceDecl(_) => {}
    }
  }
}

// visit
impl Visit for ImportExportVisitor {
  fn visit_module(&mut self, module: &ast::Module) {
    module.visit_children_with(self);
  }

  // normal
  fn visit_module_decl(&mut self, decl: &ast::ModuleDecl) {
    match decl {
      // export
      // export { a , b as c }
      // export type { a } from 'b'
      // export { a, type b } from 'b'
      // export type * as all from 'b'
      ast::ModuleDecl::ExportNamed(export) => {
        self.parse_named_export(export);
      }
      // export  default   a
      // export default []
      // export default 1
      ast::ModuleDecl::ExportDefaultExpr(export) => {
        self.parse_default_export_expr(export);
      }
      // export namespace A.B {}
      // export class A {}
      // export const a = 1
      // export enum a {}
      // export function a() {}
      // export const a = 1, b = 2
      // export type A = string
      // export interface B {}
      ast::ModuleDecl::ExportDecl(export) => {
        let need_eager_return = self.parse_export_decl(export);
        if need_eager_return {
          // skip visit children
          return;
        }
      }
      // export * from 'vv'
      ast::ModuleDecl::ExportAll(_) => {}
      // export default function a () {}
      ast::ModuleDecl::ExportDefaultDecl(export) => {
        self.parse_export_default_decl(export);
      }
      ast::ModuleDecl::Import(_) => {}
      // export = a
      // not support
      ast::ModuleDecl::TsExportAssignment(_) => {}
      // export as namespace a
      ast::ModuleDecl::TsNamespaceExport(_) => {}
      // import TypeScript = TypeScriptServices.TypeScript;
      ast::ModuleDecl::TsImportEquals(_) => {}
    };
    decl.visit_children_with(self)
  }
}
