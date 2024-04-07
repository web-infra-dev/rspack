pub mod hot_module_replacement {
  pub use super::ImportMetaHotReplacementParserPlugin;
  pub use super::ModuleHotReplacementParserPlugin;
}

use rspack_core::{BoxDependency, ErrorSpan, SpanExt};
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{CallExpr, Expr, Lit};
use swc_core::ecma::atoms::Atom;

use crate::dependency::{
  HarmonyAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
  ModuleArgumentDependency, ModuleHotAcceptDependency, ModuleHotDeclineDependency,
};
use crate::parser_plugin::JavascriptParserPlugin;
use crate::utils::eval;
use crate::visitors::{expr_name, JavascriptParser};

type CreateDependency = fn(u32, u32, Atom, Option<ErrorSpan>) -> BoxDependency;

fn extract_deps(call_expr: &CallExpr, create_dependency: CreateDependency) -> Vec<BoxDependency> {
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

  dependencies
}

impl JavascriptParser<'_> {
  fn create_hmr_expression_handler(&mut self, span: Span) {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        span.real_lo(),
        span.real_hi(),
        Some("hot"),
      )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        call_expr.callee.span().real_lo(),
        call_expr.callee.span().real_hi(),
        Some("hot.accept"),
      )));
    let dependencies = extract_deps(call_expr, create_dependency);
    if self.build_meta.esm && !call_expr.args.is_empty() {
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
    self.walk_expr_or_spread(&call_expr.args);
    Some(true)
  }

  fn create_decline_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        call_expr.callee.span().real_lo(),
        call_expr.callee.span().real_hi(),
        Some("hot.decline"),
      )));
    let dependencies = extract_deps(call_expr, create_dependency);
    self.dependencies.extend(dependencies);
    Some(true)
  }
}

pub struct ModuleHotReplacementParserPlugin;

impl JavascriptParserPlugin for ModuleHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if ident == expr_name::MODULE_HOT {
      Some(eval::evaluate_to_identifier(
        expr_name::MODULE_HOT.to_string(),
        expr_name::MODULE.to_string(),
        Some(true),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE_HOT {
      parser.create_hmr_expression_handler(expr.span());
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE_HOT_ACCEPT {
      parser.create_accept_handler(call_expr, |start, end, request, span| {
        Box::new(ModuleHotAcceptDependency::new(start, end, request, span))
      })
    } else if for_name == expr_name::MODULE_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |start, end, request, span| {
        Box::new(ModuleHotDeclineDependency::new(start, end, request, span))
      })
    } else {
      None
    }
  }
}

pub struct ImportMetaHotReplacementParserPlugin;

impl JavascriptParserPlugin for ImportMetaHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if ident == expr_name::IMPORT_META_WEBPACK_HOT {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_WEBPACK_HOT.to_string(),
        expr_name::IMPORT_META.to_string(),
        Some(true),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_WEBPACK_HOT {
      parser.create_hmr_expression_handler(expr.span());
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_WEBPACK_HOT_ACCEPT {
      parser.create_accept_handler(call_expr, |start, end, request, span| {
        Box::new(ImportMetaHotAcceptDependency::new(
          start, end, request, span,
        ))
      })
    } else if for_name == expr_name::IMPORT_META_WEBPACK_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |start, end, request, span| {
        Box::new(ImportMetaHotDeclineDependency::new(
          start, end, request, span,
        ))
      })
    } else {
      None
    }
  }
}
