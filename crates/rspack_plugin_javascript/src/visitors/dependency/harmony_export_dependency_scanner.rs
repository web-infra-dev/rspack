use rspack_core::{
  tree_shaking::symbol::DEFAULT_JS_WORD, BoxDependency, BoxDependencyTemplate, ConstDependency,
  SpanExt,
};
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{
      ClassDecl, Decl, DefaultDecl, ExportDecl, ExportDefaultDecl, ExportDefaultExpr,
      ExportSpecifier, FnDecl, Ident, ModuleExportName, NamedExport, Program,
    },
    atoms::JsWord,
    utils::find_pat_ids,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::harmony_import_dependency_scanner::ImportMap;
use crate::dependency::{
  AnonymousFunctionRangeInfo, HarmonyExportHeaderDependency,
  HarmonyExportImportedSpecifierDependency, HarmonyExportSpecifierDependency,
  HarmonyExpressionHeaderDependency, DEFAULT_EXPORT,
};

pub struct HarmonyExportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
  pub import_map: &'a mut ImportMap,
  pub exports: Vec<(JsWord, JsWord)>,
}

impl<'a> HarmonyExportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
    import_map: &'a mut ImportMap,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      import_map,
      exports: Default::default(),
    }
  }
}

impl Visit for HarmonyExportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &'_ Program) {
    program.visit_children_with(self);
    if !self.exports.is_empty() {
      self
        .presentational_dependencies
        .push(Box::new(HarmonyExportSpecifierDependency::new(
          std::mem::take(&mut self.exports),
        )));
    }
  }

  fn visit_export_decl(&mut self, export_decl: &'_ ExportDecl) {
    match &export_decl.decl {
      Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
        self.exports.push((ident.sym.clone(), ident.sym.clone()));
      }
      Decl::Var(v) => {
        self.exports.extend(
          find_pat_ids::<_, Ident>(&v.decls)
            .into_iter()
            .map(|ident| (ident.sym.clone(), ident.sym)),
        );
      }
      _ => {}
    }
    self
      .presentational_dependencies
      .push(Box::new(HarmonyExportHeaderDependency::new(
        export_decl.span().real_lo(),
      )));
  }

  fn visit_named_export(&mut self, named_export: &'_ NamedExport) {
    if named_export.src.is_none() {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              let export = match &named.exported {
                Some(ModuleExportName::Ident(export)) => export.sym.clone(),
                None => orig.sym.clone(),
                _ => unreachable!(),
              };
              if let Some(reference) = self.import_map.get(&orig.to_id()) {
                self
                  .dependencies
                  .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
                    reference.request.clone(),
                    vec![(export, reference.names.clone())],
                  )));
              } else {
                self.exports.push((export, orig.sym.clone()));
              }
            }
          }
          _ => unreachable!(),
        });
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          named_export.span.real_lo(),
          named_export.span.real_hi(),
          "".into(),
          None,
        )));
    }
  }

  fn visit_export_default_expr(&mut self, export_default_expr: &'_ ExportDefaultExpr) {
    self
      .exports
      .push((DEFAULT_JS_WORD.clone(), DEFAULT_EXPORT.into()));

    self
      .presentational_dependencies
      .push(Box::new(HarmonyExpressionHeaderDependency::new(
        export_default_expr.span().real_lo(),
        export_default_expr.expr.span().real_lo(),
        false,
        None,
      )));
  }

  fn visit_export_default_decl(&mut self, export_default_decl: &'_ ExportDefaultDecl) {
    let ident = match &export_default_decl.decl {
      DefaultDecl::Class(class_expr) => &class_expr.ident,
      DefaultDecl::Fn(f) => &f.ident,
      _ => unreachable!(),
    };

    self.exports.push((
      DEFAULT_JS_WORD.clone(),
      match &ident {
        Some(ident) => ident.sym.clone(),
        None => DEFAULT_EXPORT.into(),
      },
    ));

    self
      .presentational_dependencies
      .push(Box::new(HarmonyExpressionHeaderDependency::new(
        export_default_decl.span().real_lo(),
        export_default_decl.decl.span().real_lo(),
        ident.is_some(),
        if let DefaultDecl::Fn(f) = &export_default_decl.decl && f.ident.is_none() {
          let first_parmas_start = f.function.params.get(0).map(|first| first.span.real_lo());
          Some(AnonymousFunctionRangeInfo { is_async: f.function.is_async, is_generator:f.function.is_generator, body_start: f.function.body.span().real_lo(), first_parmas_start })
        } else {
          None
        },
      )));
  }
}
