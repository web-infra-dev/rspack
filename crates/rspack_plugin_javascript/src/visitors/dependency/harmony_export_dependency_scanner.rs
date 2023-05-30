use rspack_core::{
  get_import_var, CodeReplaceSourceDependency, ModuleDependency, ReplaceConstDependency, SpanExt,
};
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{
      ClassDecl, Decl, DefaultDecl, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr,
      ExportSpecifier, FnDecl, Ident, ModuleExportName, NamedExport, Program,
    },
    utils::find_pat_ids,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::harmony_import_dependency_scanner::ImportMap;
use crate::dependency::{
  get_reexport_var, HarmonyExportHeaderDependency, HarmonyExportSpecifierDependency,
  HarmonyExpressionHeaderDependency, HarmonyImportDependency, DEFAULT_EXPORT,
};

pub struct HarmonyExportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  pub import_map: &'a mut ImportMap,
  pub exports: Vec<(String, String)>,
  pub exports_all: Vec<String>,
}

impl<'a> HarmonyExportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
    import_map: &'a mut ImportMap,
  ) -> Self {
    Self {
      dependencies,
      code_generable_dependencies,
      import_map,
      exports: Default::default(),
      exports_all: Default::default(),
    }
  }
}

impl Visit for HarmonyExportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &'_ Program) {
    program.visit_children_with(self);
    if !self.exports.is_empty() || !self.exports_all.is_empty() {
      self
        .code_generable_dependencies
        .push(Box::new(HarmonyExportSpecifierDependency::new(
          std::mem::take(&mut self.exports),
          std::mem::take(&mut self.exports_all),
        )));
    }
  }

  fn visit_export_decl(&mut self, export_decl: &'_ ExportDecl) {
    match &export_decl.decl {
      Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
        self
          .exports
          .push((ident.sym.to_string(), ident.sym.to_string()));
      }
      Decl::Var(v) => {
        self.exports.extend(
          find_pat_ids::<_, Ident>(&v.decls)
            .into_iter()
            .map(|ident| (ident.sym.to_string(), ident.sym.to_string())),
        );
      }
      _ => {}
    }
    self
      .code_generable_dependencies
      .push(Box::new(HarmonyExportHeaderDependency::new(
        export_decl.span().real_lo(),
      )));
  }

  fn visit_named_export(&mut self, named_export: &'_ NamedExport) {
    if let Some(src) = &named_export.src {
      let import_var: String = get_import_var(&src.value);
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(ns) => {
            self.exports.push((
              match &ns.name {
                ModuleExportName::Ident(name) => name.sym.to_string(),
                _ => unreachable!(),
              },
              import_var.to_string(),
            ));
          }
          ExportSpecifier::Default(_) => {
            unreachable!();
          }
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              self.exports.push((
                match &named.exported {
                  Some(ModuleExportName::Ident(export)) => export.sym.to_string(),
                  None => orig.sym.to_string(),
                  _ => unreachable!(),
                },
                get_reexport_var(&import_var, &orig.sym),
              ));
            }
          }
        });

      // self.code_generable_dependencies.push(Box::new(
      //   HarmonyExportImportedSpecifierDependency::new(src.value.to_string()),
      // ));
      self
        .dependencies
        .push(Box::new(HarmonyImportDependency::new(
          src.value.clone(),
          Some(named_export.span.into()),
          vec![],
        )));
    } else {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(_ns) => {
            unreachable!()
          }
          ExportSpecifier::Default(_default) => {
            unreachable!();
          }
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              if self.import_map.get(&orig.to_id()).is_none() {
                self.exports.push((
                  match &named.exported {
                    Some(ModuleExportName::Ident(export)) => export.sym.to_string(),
                    None => orig.sym.to_string(),
                    _ => unreachable!(),
                  },
                  orig.sym.to_string(),
                ));
              }
            }
          }
        });
    }
    self
      .code_generable_dependencies
      .push(Box::new(ReplaceConstDependency::new(
        named_export.span.real_lo(),
        named_export.span.real_hi(),
        "".into(),
        None,
      )));
  }

  fn visit_export_all(&mut self, export_all: &'_ ExportAll) {
    let import_var = get_import_var(&export_all.src.value);
    self.exports_all.push(import_var);
    // self
    //   .code_generable_dependencies
    //   .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
    //     export_all.src.value.to_string(),
    //   )));
    self
      .dependencies
      .push(Box::new(HarmonyImportDependency::new(
        export_all.src.value.clone(),
        Some(export_all.span.into()),
        vec![],
      )));
    self
      .code_generable_dependencies
      .push(Box::new(ReplaceConstDependency::new(
        export_all.span.real_lo(),
        export_all.span.real_hi(),
        "".into(),
        None,
      )));
  }

  fn visit_export_default_expr(&mut self, export_default_expr: &'_ ExportDefaultExpr) {
    self
      .exports
      .push(("default".to_string(), DEFAULT_EXPORT.to_string()));

    self
      .code_generable_dependencies
      .push(Box::new(HarmonyExpressionHeaderDependency::new(
        export_default_expr.span().real_lo(),
        export_default_expr.expr.span().real_lo(),
        false,
      )));
  }

  fn visit_export_default_decl(&mut self, export_default_decl: &'_ ExportDefaultDecl) {
    let ident = match &export_default_decl.decl {
      DefaultDecl::Class(class_expr) => &class_expr.ident,
      DefaultDecl::Fn(f) => &f.ident,
      _ => unreachable!(),
    };

    self.exports.push((
      "default".to_string(),
      match &ident {
        Some(ident) => ident.sym.to_string(),
        None => DEFAULT_EXPORT.to_string(),
      },
    ));

    self
      .code_generable_dependencies
      .push(Box::new(HarmonyExpressionHeaderDependency::new(
        export_default_decl.span().real_lo(),
        export_default_decl.decl.span().real_lo(),
        ident.is_some(),
      )));
  }
}
