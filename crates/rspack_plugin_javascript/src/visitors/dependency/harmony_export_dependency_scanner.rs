use rspack_core::{
  tree_shaking::symbol::DEFAULT_JS_WORD, BoxDependency, BoxDependencyTemplate, BuildInfo,
  ConstDependency, DependencyLocation, SpanExt,
};
use rustc_hash::FxHashMap as HashMap;
use swc_core::{
  common::{Span, Spanned},
  ecma::{
    ast::{
      ClassDecl, Decl, DefaultDecl, ExportDecl, ExportDefaultDecl, ExportDefaultExpr,
      ExportSpecifier, FnDecl, Ident, ModuleExportName, NamedExport, Program,
    },
    utils::find_pat_ids,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};
use swc_node_comments::SwcComments;

use super::{harmony_import_dependency_scanner::ImportMap, ExtraSpanInfo};
use crate::{
  dependency::{
    AnonymousFunctionRangeInfo, HarmonyExportExpressionDependency, HarmonyExportHeaderDependency,
    HarmonyExportImportedSpecifierDependency, HarmonyExportSpecifierDependency, Specifier,
    DEFAULT_EXPORT,
  },
  no_visit_ignored_stmt,
};

pub struct HarmonyExportDependencyScanner<'a, 'b> {
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
  pub import_map: &'a mut ImportMap,
  pub build_info: &'a mut BuildInfo,
  pub rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  pub comments: Option<&'b SwcComments>,
  pub ignored: &'a mut Vec<DependencyLocation>,
}

impl<'a, 'b> HarmonyExportDependencyScanner<'a, 'b> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
    import_map: &'a mut ImportMap,
    build_info: &'a mut BuildInfo,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
    comments: Option<&'b SwcComments>,
    ignored: &'a mut Vec<DependencyLocation>,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      import_map,
      build_info,
      rewrite_usage_span,
      comments,
      ignored,
    }
  }
}

impl<'a, 'b> Visit for HarmonyExportDependencyScanner<'a, 'b> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_program(&mut self, program: &'_ Program) {
    program.visit_children_with(self);
  }

  fn visit_export_decl(&mut self, export_decl: &'_ ExportDecl) {
    match &export_decl.decl {
      Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
        self
          .dependencies
          .push(Box::new(HarmonyExportSpecifierDependency::new(
            ident.sym.clone(),
            ident.sym.clone(),
          )));

        self.rewrite_usage_span.insert(
          export_decl.span(),
          ExtraSpanInfo::AddVariableUsage(vec![(ident.sym.clone(), ident.sym.clone())]),
        );
        self
          .build_info
          .harmony_named_exports
          .insert(ident.sym.clone());
      }
      Decl::Var(v) => {
        let mut usages = vec![];
        find_pat_ids::<_, Ident>(&v.decls)
          .into_iter()
          .for_each(|ident| {
            self
              .dependencies
              .push(Box::new(HarmonyExportSpecifierDependency::new(
                ident.sym.clone(),
                ident.sym.clone(),
              )));

            usages.push((ident.sym.clone(), ident.sym.clone()));
            self.build_info.harmony_named_exports.insert(ident.sym);
          });

        self
          .rewrite_usage_span
          .insert(export_decl.span(), ExtraSpanInfo::AddVariableUsage(usages));
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
      let mut usages = vec![];
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
                let ids = vec![(export.clone(), reference.names.clone())];
                // dbg!(&reference);
                let mode_ids = match reference.specifier {
                  Specifier::Namespace(_) => {
                    vec![]
                  }
                  _ => ids.clone(),
                };
                self
                  .dependencies
                  .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
                    reference.request.clone(),
                    reference.source_order,
                    ids,
                    mode_ids,
                    Some(export.clone()),
                    false,
                    None,
                  )));
              } else {
                self
                  .dependencies
                  .push(Box::new(HarmonyExportSpecifierDependency::new(
                    export.clone(),
                    orig.sym.clone(),
                  )));

                self.build_info.harmony_named_exports.insert(export.clone());
              }
              usages.push((orig.sym.clone(), export));
            }
          }
          _ => unreachable!(),
        });

      self
        .rewrite_usage_span
        .insert(named_export.span(), ExtraSpanInfo::AddVariableUsage(usages));
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
    // TODO this should be at `HarmonyExportExpressionDependency`
    self
      .dependencies
      .push(Box::new(HarmonyExportSpecifierDependency::new(
        DEFAULT_JS_WORD.clone(),
        DEFAULT_EXPORT.into(),
      )));

    self.rewrite_usage_span.insert(
      export_default_expr.span,
      ExtraSpanInfo::AddVariableUsage(vec![(DEFAULT_EXPORT.into(), DEFAULT_JS_WORD.clone())]),
    );
    let end = self
      .comments
      .and_then(|comments| {
        let comment_list = comments.leading.get(&export_default_expr.expr.span_lo())?;
        let first_comment = comment_list.first()?;
        Some(first_comment.span.span().real_lo())
      })
      .unwrap_or(export_default_expr.expr.span().real_lo());
    self
      .presentational_dependencies
      .push(Box::new(HarmonyExportExpressionDependency::new(
        export_default_expr.span().real_lo(),
        end,
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

    // TODO this should be at `HarmonyExportExpressionDependency`

    let local = match &ident {
      Some(ident) => ident.sym.clone(),
      None => DEFAULT_EXPORT.into(),
    };
    self
      .dependencies
      .push(Box::new(HarmonyExportSpecifierDependency::new(
        DEFAULT_JS_WORD.clone(),
        local.clone(),
      )));
    self.rewrite_usage_span.insert(
      export_default_decl.span,
      ExtraSpanInfo::AddVariableUsage(vec![(local, DEFAULT_JS_WORD.clone())]),
    );
    self
      .presentational_dependencies
      .push(Box::new(HarmonyExportExpressionDependency::new(
        export_default_decl.span().real_lo(),
        export_default_decl.decl.span().real_lo(),
        ident.is_some(),
        if let DefaultDecl::Fn(f) = &export_default_decl.decl
          && f.ident.is_none()
        {
          let first_parmas_start = f.function.params.first().map(|first| first.span.real_lo());
          Some(AnonymousFunctionRangeInfo {
            is_async: f.function.is_async,
            is_generator: f.function.is_generator,
            body_start: f.function.body.span().real_lo(),
            first_parmas_start,
          })
        } else {
          None
        },
      )));
  }
}
