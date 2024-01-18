use rspack_core::{
  BoxDependency, BoxDependencyTemplate, BuildMeta, DependencyLocation, ErrorSpan, SpanExt,
};
use rustc_hash::FxHashSet;
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{CallExpr, Expr, Lit},
    atoms::Atom,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{expr_matcher, is_module_hot_accept_call, is_module_hot_decline_call};
use crate::{
  dependency::{
    HarmonyAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ModuleArgumentDependency, ModuleHotAcceptDependency, ModuleHotDeclineDependency,
  },
  no_visit_ignored_stmt,
  visitors::{is_import_meta_hot_accept_call, is_import_meta_hot_decline_call},
};

pub struct HotModuleReplacementScanner<'a> {
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
  pub build_meta: &'a BuildMeta,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

type CreateDependency = fn(u32, u32, Atom, Option<ErrorSpan>) -> BoxDependency;

impl<'a> HotModuleReplacementScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
    build_meta: &'a BuildMeta,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      build_meta,
      ignored,
    }
  }

  pub fn collect_dependencies(
    &mut self,
    call_expr: &CallExpr,
    kind: &str,
    create_dependency: CreateDependency,
  ) {
    let mut dependencies: Vec<BoxDependency> = vec![];

    if let Some(first_arg) = call_expr.args.first() {
      match &*first_arg.expr {
        Expr::Lit(Lit::Str(s)) => {
          dependencies.push(create_dependency(
            s.span.real_lo(),
            s.span.real_hi(),
            s.value.clone(),
            Some(s.span.into()),
          ));
        }
        Expr::Array(array_lit) => {
          array_lit.elems.iter().for_each(|e| {
            if let Some(expr) = e {
              if let Expr::Lit(Lit::Str(s)) = &*expr.expr {
                dependencies.push(create_dependency(
                  s.span.real_lo(),
                  s.span.real_hi(),
                  s.value.clone(),
                  Some(s.span.into()),
                ));
              }
            }
          });
        }
        _ => {}
      }
    }

    if self.build_meta.esm && kind == "accept" && !call_expr.args.is_empty() {
      let dependency_ids = dependencies.iter().map(|dep| *dep.id()).collect::<Vec<_>>();
      if let Some(callback_arg) = call_expr.args.get(1) {
        self
          .presentational_dependencies
          .push(Box::new(HarmonyAcceptDependency::new(
            callback_arg.span().real_lo(),
            callback_arg.span().real_hi(),
            true,
            dependency_ids,
          )));
      } else {
        self
          .presentational_dependencies
          .push(Box::new(HarmonyAcceptDependency::new(
            call_expr.span().real_hi() - 1,
            0,
            false,
            dependency_ids,
          )));
      }
    }

    self.dependencies.extend(dependencies);
  }
}

impl<'a> Visit for HotModuleReplacementScanner<'a> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_hot(expr) || expr_matcher::is_import_meta_webpack_hot(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ModuleArgumentDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          Some("hot"),
        )));
    }
    expr.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if is_module_hot_accept_call(call_expr) {
      self.collect_dependencies(call_expr, "accept", |start, end, request, span| {
        Box::new(ModuleHotAcceptDependency::new(start, end, request, span))
      });
    } else if is_module_hot_decline_call(call_expr) {
      self.collect_dependencies(call_expr, "decline", |start, end, request, span| {
        Box::new(ModuleHotDeclineDependency::new(start, end, request, span))
      });
    } else if is_import_meta_hot_accept_call(call_expr) {
      self.collect_dependencies(call_expr, "accept", |start, end, request, span| {
        Box::new(ImportMetaHotAcceptDependency::new(
          start, end, request, span,
        ))
      });
    } else if is_import_meta_hot_decline_call(call_expr) {
      self.collect_dependencies(call_expr, "decline", |start, end, request, span| {
        Box::new(ImportMetaHotDeclineDependency::new(
          start, end, request, span,
        ))
      });
    }
    call_expr.visit_children_with(self);
  }
}
