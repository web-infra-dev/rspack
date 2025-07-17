use rspack_core::{
  ConstDependency, ContextDependency, ContextMode, ContextNameSpaceObject, ContextOptions,
  DependencyCategory, DependencyLocation, DependencyRange, SpanExt,
};
use rspack_error::{DiagnosticExt, Severity};
use swc_core::{
  common::{Span, Spanned},
  ecma::ast::{
    AssignExpr, AssignTarget, CallExpr, Expr, ExprOrSpread, Ident, MemberExpr, NewExpr,
    SimpleAssignTarget, UnaryExpr,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    local_module_dependency::LocalModuleDependency, CommonJsFullRequireDependency,
    CommonJsRequireContextDependency, CommonJsRequireDependency, RequireHeaderDependency,
    RequireResolveContextDependency, RequireResolveDependency, RequireResolveHeaderDependency,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    context_reg_exp, create_context_dependency, create_traceable_error, expr_matcher, expr_name,
    extract_require_call_info, is_require_call_start, JavascriptParser,
  },
};

fn create_commonjs_require_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  call_expr: &CallExpr,
  arg_expr: &Expr,
) -> CommonJsRequireContextDependency {
  let result = create_context_dependency(param, parser);

  let span = call_expr.span();
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
    start: span.real_lo(),
    end: span.real_hi(),
    referenced_exports: None,
    attributes: None,
  };
  let mut dep = CommonJsRequireContextDependency::new(
    options,
    DependencyLocation::from_span(&span, &parser.source_map),
    call_expr.span().into(),
    Some(arg_expr.span().into()),
    parser.in_try,
  );
  *dep.critical_mut() = result.critical;
  dep
}

fn create_require_resolve_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  range: DependencyRange,
  weak: bool,
) -> RequireResolveContextDependency {
  let start = range.start;
  let end = range.end;

  let result = create_context_dependency(param, parser);

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

enum CallOrNewExpr<'a> {
  Call(&'a CallExpr),
  New(&'a NewExpr),
}

impl CallOrNewExpr<'_> {
  pub fn callee(&self) -> Option<&Expr> {
    match self {
      CallOrNewExpr::Call(call_expr) => call_expr.callee.as_expr().map(|e| &**e),
      CallOrNewExpr::New(new_expr) => Some(&new_expr.callee),
    }
  }

  pub fn args(&self) -> Option<&[ExprOrSpread]> {
    match self {
      CallOrNewExpr::Call(call_expr) => Some(&call_expr.args),
      CallOrNewExpr::New(new_expr) => new_expr.args.as_deref(),
    }
  }

  pub fn span(&self) -> Span {
    match self {
      CallOrNewExpr::Call(call_expr) => call_expr.span,
      CallOrNewExpr::New(new_expr) => new_expr.span,
    }
  }
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
          self.process_resolve_context(parser, option, weak);
        }
      }
      parser.dependencies.push(require_resolve_header_dependency);
    } else {
      if !self.process_resolve_item(parser, &param, weak) {
        self.process_resolve_context(parser, &param, weak);
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
    weak: bool,
  ) {
    let (start, end) = param.range();
    let dep =
      create_require_resolve_context_dependency(parser, param, (start, end - 1).into(), weak);

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
    let dep = create_commonjs_require_context_dependency(parser, param, call_expr, argument_expr);
    parser.dependencies.push(Box::new(dep));
    Some(true)
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    expr: CallOrNewExpr,
    for_name: &str,
  ) -> Option<bool> {
    let callee = expr.callee()?;
    let is_require_expr = for_name == expr_name::REQUIRE || expr_matcher::is_module_require(callee); // FIXME: remove `module.require`
    let args = expr.args()?;

    if !is_require_expr || args.len() != 1 {
      return None;
    }

    let argument_expr = &args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self.process_require_item(parser, expr.span(), p).is_none() {
          is_expression = true;
        }
      }
      if !is_expression {
        let range: DependencyRange = callee.span().into();
        parser
          .presentational_dependencies
          .push(Box::new(RequireHeaderDependency::new(
            range,
            Some(parser.source_map.clone()),
          )));
        return Some(true);
      }
    }
    if param.is_string()
      && let Some(local_module) = parser.get_local_module_mut(param.string())
    {
      local_module.flag_used();
      let span = expr.span();
      let dep = Box::new(LocalModuleDependency::new(
        local_module.clone(),
        Some(span.into()),
        matches!(expr, CallOrNewExpr::New(_)),
      ));
      parser.presentational_dependencies.push(dep);
      return Some(true);
    }

    if matches!(parser.javascript_options.require_dynamic, Some(false)) && !param.is_string() {
      return None;
    }

    if self
      .process_require_item(parser, expr.span(), &param)
      .is_none()
      && let CallOrNewExpr::Call(call_expr) = expr
    {
      self.process_require_context(parser, call_expr, &param);
    } else {
      let range: DependencyRange = callee.span().into();
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

    let span = ident.span();
    let start = span.real_lo();
    let end = span.real_hi();
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
      DependencyLocation::from_span(&span, &parser.source_map),
      ident.span().into(),
      None,
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
  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
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
          expr.span().into(),
          "undefined".into(),
          None,
        )));
      Some(false)
    } else {
      None
    }
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
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
  ) -> Option<BasicEvaluatedExpression<'static>> {
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
          expr.span().into(),
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
      .require_handler(parser, CallOrNewExpr::Call(call_expr), for_name)
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

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    new_expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self
      .require_handler(parser, CallOrNewExpr::New(new_expr), for_name)
      .unwrap_or_default()
    {
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

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    expr: &AssignExpr,
    _for_name: Option<&str>,
  ) -> Option<bool> {
    let AssignTarget::Simple(SimpleAssignTarget::Ident(left_expr)) = &expr.left else {
      return None;
    };

    if left_expr.sym == "require" && parser.is_unresolved_ident("require") {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          (0, 0).into(),
          "var require;".into(),
          None,
        )));
      return Some(true);
    }

    None
  }
}
