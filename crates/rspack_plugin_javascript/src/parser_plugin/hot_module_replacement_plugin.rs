pub mod hot_module_replacement {
  pub use super::ImportMetaHotReplacementParserPlugin;
  pub use super::ModuleHotReplacementParserPlugin;
}

use rspack_core::{BoxDependency, RealDependencyLocation, RealDependencyLocation, SpanExt};
use rspack_error::ErrorLocation;
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

type CreateDependency = fn(Atom, RealDependencyLocation) -> BoxDependency;

fn extract_deps(call_expr: &CallExpr, create_dependency: CreateDependency) -> Vec<BoxDependency> {
  let mut dependencies: Vec<BoxDependency> = vec![];

  if let Some(first_arg) = call_expr.args.first() {
    match &*first_arg.expr {
      Expr::Lit(Lit::Str(s)) => {
        dependencies.push(create_dependency(s.value.clone(), s.span.into()));
      }
      Expr::Array(array_lit) => {
        array_lit.elems.iter().for_each(|e| {
          if let Some(expr) = e {
            if let Expr::Lit(Lit::Str(s)) = &*expr.expr {
              dependencies.push(create_dependency(s.value.clone(), s.span.into()));
            }
          }
        });
      }
      _ => {}
    }
  }

  dependencies
}

impl<'parser> JavascriptParser<'parser> {
  fn create_hmr_expression_handler(&mut self, span: Span) {
    let range: RealDependencyLocation = span.into();
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        Some("hot"),
        range.with_source(self.source_map.clone()),
      )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    let range: RealDependencyLocation = call_expr.callee.span().into();
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        Some("hot.accept"),
        range.with_source(self.source_map.clone()),
      )));
    let dependencies = extract_deps(call_expr, create_dependency);
    if self.build_meta.esm && !call_expr.args.is_empty() {
      let dependency_ids = dependencies.iter().map(|dep| *dep.id()).collect::<Vec<_>>();
      let callback_arg = call_expr.args.get(1);
      let range = if let Some(callback) = callback_arg {
        Into::<RealDependencyLocation>::into(callback.span())
      } else {
        RealDependencyLocation::new(call_expr.span().real_hi() - 1, 0)
      };
      self
        .presentational_dependencies
        .push(Box::new(HarmonyAcceptDependency::new(
          range.with_source(self.source_map.clone()),
          callback_arg.is_some(),
          dependency_ids,
        )));
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
    let range: RealDependencyLocation = call_expr.callee.span().into();
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self
      .presentational_dependencies
      .push(Box::new(ModuleArgumentDependency::new(
        Some("hot.decline"),
        range.with_source(self.source_map.clone()),
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
      parser.create_accept_handler(call_expr, |request, range| {
        Box::new(ModuleHotAcceptDependency::new(request, range))
      })
    } else if for_name == expr_name::MODULE_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |request, range| {
        Box::new(ModuleHotDeclineDependency::new(request, range))
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
      parser.create_accept_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotAcceptDependency::new(request, range))
      })
    } else if for_name == expr_name::IMPORT_META_WEBPACK_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotDeclineDependency::new(request, range))
      })
    } else {
      None
    }
  }
}
