use std::sync::Arc;

use rspack_core::{BoxDependency, DependencyRange, ImportMetaKnownProperties};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{ArgumentData, CallExpression, GetSpan, Span};

use crate::{
  Atom,
  dependency::{
    ESMAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ModuleArgumentDependency, ModuleHotAcceptDependency, ModuleHotDeclineDependency,
    import_emitted_runtime,
  },
  parser_plugin::JavascriptParserPlugin,
  utils::eval,
  visitors::{HookMemberExpression, JavascriptParser, expr_name},
};

type CreateDependency = fn(Atom, DependencyRange) -> BoxDependency;

fn extract_deps(
  parser: &mut JavascriptParser,
  call_expr: CallExpression,
  create_dependency: CreateDependency,
) -> Vec<BoxDependency> {
  let mut dependencies: Vec<BoxDependency> = vec![];
  let ast = parser.ast.ast;

  if let Some(first_arg) = call_expr.arguments(ast).get_node(ast, 0)
    && let ArgumentData::Expr(first_arg) = ast.argument_data(first_arg)
  {
    let expr = parser.evaluate_expression(first_arg);
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
    let range = DependencyRange::from(span);
    let loc = self.to_dependency_location(range);
    self.add_presentational_dependency(Arc::new(ModuleArgumentDependency::new(
      Some("hot".into()),
      span.into(),
      loc,
    )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: CallExpression,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    let ast = self.ast.ast;
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    let callee_span = call_expr.callee(ast).span(ast);
    let callee_range = DependencyRange::from(callee_span);
    let loc = self.to_dependency_location(callee_range);
    self.add_presentational_dependency(Arc::new(ModuleArgumentDependency::new(
      Some("hot.accept".into()),
      callee_span.into(),
      loc,
    )));
    let dependencies = extract_deps(self, call_expr, create_dependency);
    if !dependencies.is_empty() {
      let dependency_ids = dependencies.iter().map(|dep| *dep.id()).collect::<Vec<_>>();
      let callback_arg = call_expr.arguments(ast).get_node(ast, 1);
      let range = if let Some(callback) = callback_arg {
        Into::<DependencyRange>::into(callback.span(ast))
      } else {
        DependencyRange::new(call_expr.span(ast).real_hi() - 1, 0)
      };
      let call_range = DependencyRange::from(call_expr.span(ast));
      let loc = self.to_dependency_location(call_range);
      self.add_presentational_dependency(Arc::new(ESMAcceptDependency::new(
        range,
        callback_arg.is_some(),
        dependency_ids,
        loc,
      )));
      self.add_dependencies(dependencies);
      self.walk_arguments(
        call_expr
          .arguments(ast)
          .iter()
          .skip(1)
          .map(|id| ast.get_node_in_sub_range(id)),
      );
      return Some(true);
    }
    self.walk_arguments(
      call_expr
        .arguments(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id)),
    );
    Some(true)
  }

  fn create_decline_handler(
    &mut self,
    call_expr: CallExpression,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    let ast = self.ast.ast;
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    let callee_span = call_expr.callee(ast).span(ast);
    let callee_range = DependencyRange::from(callee_span);
    let loc = self.to_dependency_location(callee_range);
    self.add_presentational_dependency(Arc::new(ModuleArgumentDependency::new(
      Some("hot.decline".into()),
      callee_span.into(),
      loc,
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

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ModuleHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
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
    parser: &mut JavascriptParser<'p>,
    expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE_HOT {
      parser.create_hmr_expression_handler(expr.span(parser.ast.ast));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE_HOT_ACCEPT {
      parser.create_accept_handler(call_expr, |request, range| {
        BoxDependency::new(ModuleHotAcceptDependency::new(request, range))
      })
    } else if for_name == expr_name::MODULE_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |request, range| {
        BoxDependency::new(ModuleHotDeclineDependency::new(request, range))
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

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportMetaHotReplacementParserPlugin {
  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
    if for_name == expr_name::IMPORT_META_HOT
      && parser
        .javascript_options
        .import_meta()
        .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
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
    parser: &mut JavascriptParser<'p>,
    expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_HOT
      && parser
        .javascript_options
        .import_meta()
        .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      parser.create_hmr_expression_handler(expr.span(parser.ast.ast));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if !parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      return None;
    }

    if for_name == expr_name::IMPORT_META_HOT_ACCEPT {
      parser.create_accept_handler(call_expr, |request, range| {
        BoxDependency::new(ImportMetaHotAcceptDependency::new(request, range))
      })
    } else if for_name == expr_name::IMPORT_META_HOT_DECLINE {
      parser.create_decline_handler(call_expr, |request, range| {
        BoxDependency::new(ImportMetaHotDeclineDependency::new(request, range))
      })
    } else {
      None
    }
  }
}
