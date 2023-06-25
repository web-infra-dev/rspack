use rspack_core::{
  BoxModuleDependency, BuildMeta, CodeGeneratableDependency, ConstDependency, ErrorSpan,
  ModuleDependency, ModuleIdentifier, RuntimeGlobals, SpanExt,
};
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{CallExpr, Expr, Lit},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{expr_matcher, is_module_hot_accept_call, is_module_hot_decline_call};
use crate::{
  dependency::{
    HarmonyAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ModuleHotAcceptDependency, ModuleHotDeclineDependency,
  },
  visitors::{is_import_meta_hot_accept_call, is_import_meta_hot_decline_call},
};

pub struct HotModuleReplacementScanner<'a> {
  pub dependencies: &'a mut Vec<BoxModuleDependency>,
  pub presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
  pub module_identifier: ModuleIdentifier,
  pub build_meta: &'a BuildMeta,
}

type CreateDependency = fn(u32, u32, JsWord, Option<ErrorSpan>) -> BoxModuleDependency;

impl<'a> HotModuleReplacementScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxModuleDependency>,
    presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
    module_identifier: ModuleIdentifier,
    build_meta: &'a BuildMeta,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      module_identifier,
      build_meta,
    }
  }

  pub fn collect_dependencies(
    &mut self,
    call_expr: &CallExpr,
    kind: &str,
    create_dependency: CreateDependency,
  ) {
    let mut deps = vec![];

    if let Some(first_arg) = call_expr.args.get(0) {
      match &*first_arg.expr {
        Expr::Lit(Lit::Str(s)) => {
          deps.push(create_dependency(
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
                deps.push(create_dependency(
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
      let ref_deps = deps
        .iter()
        .map(|dep| {
          (
            dep.request().into(),
            *dep.category(),
            dep.dependency_type().clone(),
          )
        })
        .collect::<Vec<_>>();
      if let Some(callback_arg) = call_expr.args.get(1) {
        self
          .presentational_dependencies
          .push(Box::new(HarmonyAcceptDependency::new(
            callback_arg.span().real_lo(),
            callback_arg.span().real_hi(),
            true,
            self.module_identifier,
            ref_deps,
          )));
      } else {
        self
          .presentational_dependencies
          .push(Box::new(HarmonyAcceptDependency::new(
            call_expr.span().real_hi() - 1,
            0,
            false,
            self.module_identifier,
            ref_deps,
          )));
      }
    }

    self.dependencies.extend(deps);
  }
}

impl<'a> Visit for HotModuleReplacementScanner<'a> {
  noop_visit_type!();

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_hot(expr) || expr_matcher::is_import_meta_webpack_hot(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "module.hot".into(), // TODO module_argument
          Some(RuntimeGlobals::MODULE),
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
