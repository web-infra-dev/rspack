use rspack_core::{BoxDependency, DependencyRange};
use rspack_util::SpanExt;
use swc_core::{
  common::{Span, Spanned},
  ecma::{ast::CallExpr, atoms::Atom},
};

use crate::{
  dependency::{
    ESMAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ModuleArgumentDependency, ModuleHotAcceptDependency, ModuleHotDeclineDependency,
    import_emitted_runtime,
  },
  parser_plugin::JavascriptParserPlugin,
  utils::eval,
  visitors::{JavascriptParser, expr_name},
};

type CreateDependency = fn(Atom, DependencyRange) -> BoxDependency;

fn extract_deps(
  parser: &mut JavascriptParser,
  call_expr: &CallExpr,
  create_dependency: CreateDependency,
) -> Vec<BoxDependency> {
  let mut dependencies: Vec<BoxDependency> = vec![];

  if let Some(first_arg) = call_expr.args.first() {
    let expr = parser.evaluate_expression(&first_arg.expr);
    if expr.is_string() {
      dependencies.push(create_dependency(
        expr.string().as_str().into(),
        expr.range().into(),
      ));
    } else if expr.is_array() {
      expr
        .items()
        .iter()
        .filter(|item| item.is_string())
        .for_each(|expr| {
          dependencies.push(create_dependency(
            expr.string().as_str().into(),
            expr.range().into(),
          ));
        });
    }
  }

  dependencies
}

impl JavascriptParser<'_> {
  fn create_hmr_expression_handler(&mut self, span: Span) {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot".into()),
      span.into(),
      Some(self.source().clone()),
    )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot.accept".into()),
      call_expr.callee.span().into(),
      Some(self.source().clone()),
    )));
    let dependencies = extract_deps(self, call_expr, create_dependency);
    if !dependencies.is_empty() {
      let dependency_ids = dependencies.iter().map(|dep| *dep.id()).collect::<Vec<_>>();
      let callback_arg = call_expr.args.get(1);
      let range = if let Some(callback) = callback_arg {
        Into::<DependencyRange>::into(callback.span())
      } else {
        DependencyRange::new(call_expr.span().real_hi() - 1, 0)
      };
      self.add_presentational_dependency(Box::new(ESMAcceptDependency::new(
        range,
        callback_arg.is_some(),
        dependency_ids,
        Some(self.source().clone()),
      )));
      self.add_dependencies(dependencies);
      for arg in call_expr.args.iter().skip(1) {
        self.walk_expression(&arg.expr);
      }
      return Some(true);
    }
    self.walk_expr_or_spread(&call_expr.args);
    Some(true)
  }

  fn create_decline_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot.decline".into()),
      call_expr.callee.span().into(),
      Some(self.source().clone()),
    )));
    let dependencies = extract_deps(self, call_expr, create_dependency);
    self.add_dependencies(dependencies);
    Some(true)
  }
}

pub struct ModuleHotReplacementParserPlugin {
  _private: (),
}

impl ModuleHotReplacementParserPlugin {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    import_emitted_runtime::init_map();
    Self { _private: () }
  }
}

impl JavascriptParserPlugin for ModuleHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::MODULE_HOT {
      Some(eval::evaluate_to_identifier(
        expr_name::MODULE_HOT.into(),
        expr_name::MODULE.into(),
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

pub struct ImportMetaHotReplacementParserPlugin {
  _private: (),
}

impl ImportMetaHotReplacementParserPlugin {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    import_emitted_runtime::init_map();
    Self { _private: () }
  }
}

impl JavascriptParserPlugin for ImportMetaHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::IMPORT_META_HOT {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_HOT.into(),
        expr_name::IMPORT_META.into(),
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
    if for_name == expr_name::IMPORT_META_HOT {
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
    if for_name == expr_name::IMPORT_META_HOT_ACCEPT {
      parser.create_accept_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotAcceptDependency::new(request, range))
      })
    } else if for_name == expr_name::IMPORT_META_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotDeclineDependency::new(request, range))
      })
    } else {
      None
    }
  }
}
