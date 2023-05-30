use rspack_core::{
  BoxModuleDependency, BuildMeta, CodeReplaceSourceDependency, ErrorSpan, ModuleDependency,
  ModuleIdentifier, ReplaceConstDependency, RuntimeGlobals, SpanExt,
};
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{CallExpr, Expr, Lit},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{is_module_hot_accept_call, is_module_hot_decline_call};
use crate::{
  dependency::{
    HarmonyAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    NewModuleHotAcceptDependency, NewModuleHotDeclineDependency,
  },
  visitors::{is_import_meta_hot_accept_call, is_import_meta_hot_decline_call},
};

pub struct HotModuleReplacementScanner<'a> {
  pub dependencies: &'a mut Vec<BoxModuleDependency>,
  pub code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  pub module_identifier: ModuleIdentifier,
  pub build_meta: &'a BuildMeta,
}

type CreateDependency = fn(u32, u32, JsWord, Option<ErrorSpan>) -> BoxModuleDependency;

impl<'a> HotModuleReplacementScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxModuleDependency>,
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
    module_identifier: ModuleIdentifier,
    build_meta: &'a BuildMeta,
  ) -> Self {
    Self {
      dependencies,
      code_generable_dependencies,
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
    self
      .code_generable_dependencies
      .push(Box::new(ReplaceConstDependency::new(
        call_expr.callee.span().real_lo(),
        call_expr.callee.span().real_hi(),
        format!("module.hot.{kind}").into(),
        Some(RuntimeGlobals::MODULE),
      )));

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

    if self.build_meta.esm && kind == "accept" {
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
          .code_generable_dependencies
          .push(Box::new(HarmonyAcceptDependency::new(
            callback_arg.span().real_lo(),
            callback_arg.span().real_hi(),
            true,
            self.module_identifier,
            ref_deps,
          )));
      } else {
        self
          .code_generable_dependencies
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

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if is_module_hot_accept_call(call_expr) {
      self.collect_dependencies(call_expr, "accept", |start, end, request, span| {
        Box::new(NewModuleHotAcceptDependency::new(start, end, request, span))
      });
    } else if is_module_hot_decline_call(call_expr) {
      self.collect_dependencies(call_expr, "decline", |start, end, request, span| {
        Box::new(NewModuleHotDeclineDependency::new(
          start, end, request, span,
        ))
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
    } else {
      call_expr.visit_children_with(self);
    }
  }
}
