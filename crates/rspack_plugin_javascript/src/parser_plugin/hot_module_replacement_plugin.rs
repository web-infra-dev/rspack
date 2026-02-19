use rspack_core::{BoxDependency, DependencyRange};
use rspack_util::{SpanExt, atom::Atom};
use swc_experimental_ecma_ast::{CallExpr, GetSpan, MemberExpr, Span};

use crate::{
  dependency::{
    ESMAcceptDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ModuleArgumentDependency, ModuleHotAcceptDependency, ModuleHotDeclineDependency,
    import_emitted_runtime,
  },
  parser_plugin::JavascriptParserPlugin,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{JavascriptParser, expr_name},
};

type CreateDependency = fn(Atom, DependencyRange) -> BoxDependency;

fn extract_deps(
  parser: &mut JavascriptParser,
  call_expr: CallExpr,
  create_dependency: CreateDependency,
) -> Vec<BoxDependency> {
  let mut dependencies: Vec<BoxDependency> = vec![];

  if let Some(first_arg) = call_expr.args(&parser.ast).first() {
    let first_arg = parser.ast.get_node_in_sub_range(first_arg);
    let expr = parser.evaluate_expression(first_arg.expr(&parser.ast));
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
      Some(self.source()),
    )));
  }

  fn create_accept_handler(
    &mut self,
    call_expr: CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot.accept".into()),
      call_expr.callee(&self.ast).span(&self.ast).into(),
      Some(self.source()),
    )));
    let dependencies = extract_deps(self, call_expr, create_dependency);
    if !dependencies.is_empty() {
      let dependency_ids = dependencies.iter().map(|dep| *dep.id()).collect::<Vec<_>>();
      let callback_arg = call_expr.args(&self.ast).get(1);
      let range = if let Some(callback) = callback_arg {
        let callback = self.ast.get_node_in_sub_range(callback);
        Into::<DependencyRange>::into(callback.span(&self.ast))
      } else {
        DependencyRange::new(call_expr.span(&self.ast).real_hi() - 1, 0)
      };
      self.add_presentational_dependency(Box::new(ESMAcceptDependency::new(
        range,
        callback_arg.is_some(),
        dependency_ids,
        Some(self.source()),
      )));
      self.add_dependencies(dependencies);
      for arg in call_expr.args(&self.ast).iter().skip(1) {
        let arg = self.ast.get_node_in_sub_range(arg);
        self.walk_expression(arg.expr(&self.ast));
      }
      return Some(true);
    }
    self.walk_expr_or_spread(call_expr.args(&self.ast));
    Some(true)
  }

  fn create_decline_handler(
    &mut self,
    call_expr: CallExpr,
    create_dependency: CreateDependency,
  ) -> Option<bool> {
    self.build_info.module_concatenation_bailout = Some(String::from("Hot Module Replacement"));
    self.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
      Some("hot.decline".into()),
      call_expr.callee(&self.ast).span(&self.ast).into(),
      Some(self.source()),
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
  ) -> Option<BasicEvaluatedExpression> {
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
    expr: MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE_HOT {
      parser.create_hmr_expression_handler(expr.span(&parser.ast));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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
  ) -> Option<BasicEvaluatedExpression> {
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
    expr: MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_HOT {
      parser.create_hmr_expression_handler(expr.span(&parser.ast));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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
