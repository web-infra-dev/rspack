use rspack_core::{
  BoxDependency, ConstDependency, DependencyRange, ImportMetaKnownProperties,
};
use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CallExpr, GetSpan, MemberExpr, Span, UnaryExpr};

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

fn is_import_meta_hot(for_name: &str) -> bool {
  matches!(
    for_name,
    expr_name::IMPORT_META_HOT | expr_name::IMPORT_META_HOT_ALIAS
  )
}

fn is_import_meta_hot_accept(for_name: &str) -> bool {
  matches!(
    for_name,
    expr_name::IMPORT_META_HOT_ACCEPT | expr_name::IMPORT_META_HOT_ALIAS_ACCEPT
  )
}

fn is_import_meta_hot_decline(for_name: &str) -> bool {
  matches!(
    for_name,
    expr_name::IMPORT_META_HOT_DECLINE | expr_name::IMPORT_META_HOT_ALIAS_DECLINE
  )
}

fn evaluate_typeof_import_meta_hot(for_name: &str) -> Option<&'static str> {
  if is_import_meta_hot(for_name) {
    Some("object")
  } else if is_import_meta_hot_accept(for_name) || is_import_meta_hot_decline(for_name) {
    Some("function")
  } else {
    None
  }
}

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
    let range = DependencyRange::from(span);
    let loc = self.to_dependency_location(range);
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot".into()),
      span.into(),
      loc,
    )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: &CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    let callee_span = call_expr.callee.span();
    let callee_range = DependencyRange::from(callee_span);
    let loc = self.to_dependency_location(callee_range);
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot.accept".into()),
      callee_span.into(),
      loc,
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
      let call_range = DependencyRange::from(call_expr.span());
      let loc = self.to_dependency_location(call_range);
      self.add_presentational_dependency(Box::new(ESMAcceptDependency::new(
        range,
        callback_arg.is_some(),
        dependency_ids,
        loc,
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
    let callee_span = call_expr.callee.span();
    let callee_range = DependencyRange::from(callee_span);
    let loc = self.to_dependency_location(callee_range);
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
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
    expr: &MemberExpr,
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
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
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

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportMetaHotReplacementParserPlugin {
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'a>> {
    if !parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      return None;
    }

    evaluate_typeof_import_meta_hot(for_name).map(|res| {
      eval::evaluate_to_string(res.to_string(), expr.span.real_lo(), expr.span.real_hi())
    })
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
    if is_import_meta_hot(for_name)
      && parser
        .javascript_options
        .import_meta()
        .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      Some(eval::evaluate_to_identifier(
        for_name.into(),
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
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if is_import_meta_hot(for_name)
      && parser
        .javascript_options
        .import_meta()
        .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      parser.create_hmr_expression_handler(expr.span());
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if !parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      return None;
    }

    if is_import_meta_hot_accept(for_name) {
      parser.create_accept_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotAcceptDependency::new(request, range))
      })
    } else if is_import_meta_hot_decline(for_name) {
      parser.create_decline_handler(call_expr, |request, range| {
        Box::new(ImportMetaHotDeclineDependency::new(request, range))
      })
    } else {
      None
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if !parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
    {
      return None;
    }

    evaluate_typeof_import_meta_hot(for_name).map(|res| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        format!("'{res}'").into(),
      )));
      true
    })
  }
}
