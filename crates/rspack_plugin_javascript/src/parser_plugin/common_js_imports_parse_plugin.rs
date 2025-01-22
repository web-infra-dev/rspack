use rspack_core::{
  ConstDependency, ContextDependency, ContextMode, DependencyCategory, DependencyRange, SpanExt,
};
use rspack_core::{ContextNameSpaceObject, ContextOptions};
use rspack_error::{DiagnosticExt, Severity};
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{CallExpr, Expr, Ident, MemberExpr, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::dependency::{
  CommonJsFullRequireDependency, CommonJsRequireContextDependency, CommonJsRequireDependency,
  RequireHeaderDependency, RequireResolveContextDependency, RequireResolveDependency,
  RequireResolveHeaderDependency,
};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::{
  context_reg_exp, create_context_dependency, create_traceable_error, expr_matcher, expr_name,
  JavascriptParser,
};
use crate::visitors::{extract_require_call_info, is_require_call_start};

fn create_commonjs_require_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  expr: &Expr,
  span: Span,
  callee_span: Span,
) -> CommonJsRequireContextDependency {
  let start = callee_span.real_lo();
  let end = callee_span.real_hi();
  let result = create_context_dependency(param, expr, parser);
  let options = ContextOptions {
    mode: ContextMode::Sync,
    recursive: true,
    reg_exp: context_reg_exp(&result.reg, "", None, parser),
    include: None,
    exclude: None,
    category: DependencyCategory::CommonJS,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    namespace_object: ContextNameSpaceObject::Unset,
    group_options: None,
    replaces: result.replaces,
    start,
    end,
    referenced_exports: None,
    attributes: None,
  };
  let mut dep =
    CommonJsRequireContextDependency::new(options, span.into(), (start, end).into(), parser.in_try);
  *dep.critical_mut() = result.critical;
  dep
}

fn create_require_resolve_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  expr: &Expr,
  range: DependencyRange,
  weak: bool,
) -> RequireResolveContextDependency {
  let start = range.start;
  let end = range.end;
  let result = create_context_dependency(param, expr, parser);
  let options = ContextOptions {
    mode: if weak {
      ContextMode::Weak
    } else {
      ContextMode::Sync
    },
    recursive: true,
    reg_exp: context_reg_exp(&result.reg, "", None, parser),
    include: None,
    exclude: None,
    category: DependencyCategory::CommonJS,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    namespace_object: ContextNameSpaceObject::Unset,
    group_options: None,
    replaces: result.replaces,
    start,
    end,
    referenced_exports: None,
    attributes: None,
  };
  RequireResolveContextDependency::new(options, range, parser.in_try)
}

pub struct CommonJsImportsParserPlugin;

impl CommonJsImportsParserPlugin {
  fn process_resolve(&self, parser: &mut JavascriptParser, call_expr: &CallExpr, weak: bool) {
    if matches!(parser.javascript_options.require_resolve, Some(false)) {
      return;
    }

    if call_expr.args.len() != 1 {
      return;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    let require_resolve_header_dependency = Box::new(RequireResolveHeaderDependency::new(
      call_expr.callee.span().into(),
      Some(parser.source_map.clone()),
    ));

    if param.is_conditional() {
      for option in param.options() {
        if !self.process_resolve_item(parser, option, weak) {
          self.process_resolve_context(parser, option, argument_expr, weak);
        }
      }
      parser.dependencies.push(require_resolve_header_dependency);
    } else {
      if !self.process_resolve_item(parser, &param, weak) {
        self.process_resolve_context(parser, &param, argument_expr, weak);
      }
      parser.dependencies.push(require_resolve_header_dependency);
    }
  }

  fn process_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
    weak: bool,
  ) -> bool {
    if param.is_string() {
      let (start, end) = param.range();
      parser
        .dependencies
        .push(Box::new(RequireResolveDependency::new(
          param.string().to_string(),
          (start, end - 1).into(),
          weak,
          parser.in_try,
        )));

      return true;
    }

    false
  }

  fn process_resolve_context(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
    argument_expr: &Expr,
    weak: bool,
  ) {
    let (start, end) = param.range();
    let dep = create_require_resolve_context_dependency(
      parser,
      param,
      argument_expr,
      (start, end - 1).into(),
      weak,
    );

    parser.dependencies.push(Box::new(dep));
  }

  fn chain_handler(
    &self,
    parser: &mut JavascriptParser,
    mem_expr: &MemberExpr,
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    let expr = Expr::Member(mem_expr.to_owned());

    let is_require_member_chain = is_require_call_start(&expr)
      && !expr_matcher::is_require(&expr)
      && !expr_matcher::is_module_require(&expr)
      && parser.is_unresolved_ident("require");
    if !is_require_member_chain {
      return None;
    }

    let (members, first_arg) = extract_require_call_info(parser, mem_expr)?;

    let range: DependencyRange = mem_expr.span.into();
    let param = parser.evaluate_expression(&first_arg.expr);
    param.is_string().then(|| {
      CommonJsFullRequireDependency::new(
        param.string().to_owned(),
        members,
        range,
        is_call,
        parser.in_try,
        !parser.is_asi_position(mem_expr.span_lo()),
        Some(parser.source_map.clone()),
      )
    })
  }

  fn process_require_item(
    &self,
    parser: &mut JavascriptParser,
    span: Span,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    param.is_string().then(|| {
      let range_expr: DependencyRange = param.range().into();
      let dep = CommonJsRequireDependency::new(
        param.string().to_string(),
        range_expr,
        Some(span.into()),
        parser.in_try,
        Some(parser.source_map.clone()),
      );
      parser.dependencies.push(Box::new(dep));
      true
    })
  }

  fn process_require_context(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let Some(argument_expr) = &call_expr.args.first().map(|expr| expr.expr.as_ref()) else {
      unreachable!("ensure require includes arguments")
    };
    let dep = create_commonjs_require_context_dependency(
      parser,
      param,
      argument_expr,
      call_expr.span,
      call_expr.callee.span(),
    );
    parser.dependencies.push(Box::new(dep));
    Some(true)
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    let is_require_expr = for_name == expr_name::REQUIRE
      || call_expr
        .callee
        .as_expr()
        .is_some_and(|expr| expr_matcher::is_module_require(&**expr)); // FIXME: remove `module.require`

    if !is_require_expr || call_expr.args.len() != 1 {
      return None;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self
          .process_require_item(parser, call_expr.span(), p)
          .is_none()
        {
          is_expression = true;
        }
      }
      if !is_expression {
        let range: DependencyRange = call_expr.callee.span().into();
        parser
          .presentational_dependencies
          .push(Box::new(RequireHeaderDependency::new(
            range,
            Some(parser.source_map.clone()),
          )));
        return Some(true);
      }
    }

    if matches!(parser.javascript_options.require_dynamic, Some(false)) && !param.is_string() {
      return None;
    }

    // FIXME: should support `LocalModuleDependency`
    if self
      .process_require_item(parser, call_expr.span, &param)
      .is_none()
    {
      self.process_require_context(parser, call_expr, &param);
    } else {
      let range: DependencyRange = call_expr.callee.span().into();
      parser
        .presentational_dependencies
        .push(Box::new(RequireHeaderDependency::new(
          range,
          Some(parser.source_map.clone()),
        )));
    }
    Some(true)
  }

  fn require_as_expression_handler(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
  ) -> Option<bool> {
    if parser.javascript_options.require_as_expression == Some(false) {
      return None;
    }

    let start = ident.span().real_lo();
    let end = ident.span().real_hi();
    let mut dep = CommonJsRequireContextDependency::new(
      ContextOptions {
        mode: ContextMode::Sync,
        recursive: true,
        reg_exp: None,
        include: None,
        exclude: None,
        category: DependencyCategory::Unknown,
        request: ".".to_string(),
        context: ".".to_string(),
        namespace_object: ContextNameSpaceObject::Unset,
        group_options: None,
        replaces: Vec::new(),
        start,
        end,
        referenced_exports: None,
        attributes: None,
      },
      ident.span().into(),
      (start, end).into(),
      parser.in_try,
    );
    *dep.critical_mut() = Some(
      create_traceable_error(
        "Critical dependency".into(),
        "require function is used in a way in which dependencies cannot be statically extracted"
          .to_string(),
        parser.source_file,
        ident.span().into(),
      )
      .with_severity(Severity::Warn)
      .boxed()
      .into(),
    );
    parser.dependencies.push(Box::new(dep));
    Some(true)
  }
}

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn can_rename(
    &self,
    parser: &mut JavascriptParser,
    str: &str,
    _is_parameter: bool,
  ) -> Option<bool> {
    if str == expr_name::REQUIRE && parser.is_unresolved_ident(str) {
      Some(true)
    } else {
      None
    }
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    if str == expr_name::REQUIRE && parser.is_unresolved_ident(str) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "undefined".into(),
          None,
        )));
      Some(false)
    } else {
      None
    }
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    (for_name == expr_name::REQUIRE
      || for_name == expr_name::REQUIRE_RESOLVE
      || for_name == expr_name::REQUIRE_RESOLVE_WEAK)
      .then(|| {
        eval::evaluate_to_string(
          "function".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        )
      })
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    match ident {
      expr_name::REQUIRE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE_WEAK => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE_WEAK.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      _ => None,
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    // same as webpack/tagRequireExpression
    if for_name == expr_name::REQUIRE
      || for_name == expr_name::REQUIRE_RESOLVE
      || for_name == expr_name::REQUIRE_RESOLVE_WEAK
    {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "'function'".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if let Some(dep) = call_expr
      .callee
      .as_expr()
      .and_then(|expr| expr.as_member())
      .and_then(|mem| self.chain_handler(parser, mem, true))
    {
      parser.dependencies.push(Box::new(dep));
      parser.walk_expr_or_spread(&call_expr.args);
      return Some(true);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self
      .require_handler(parser, call_expr, for_name)
      .unwrap_or_default()
    {
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE {
      self.process_resolve(parser, call_expr, false);
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE_WEAK {
      self.process_resolve(parser, call_expr, true);
      Some(true)
    } else {
      None
    }
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if let Some(dep) = self.chain_handler(parser, expr, false) {
      parser.dependencies.push(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      return self.require_as_expression_handler(parser, ident);
    }
    None
  }
}
