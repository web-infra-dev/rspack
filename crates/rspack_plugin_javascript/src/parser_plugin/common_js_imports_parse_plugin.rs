use rspack_core::{
  ConstDependency, ContextDependency, ContextMode, ContextNameSpaceObject, ContextOptions,
  DependencyCategory, DependencyRange,
};
use rspack_error::{Diagnostic, Severity};
use rspack_util::SpanExt;
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{
    AssignExpr, CallExpr, Callee, Expr, ExprOrSpread, Ident, MemberExpr, NewExpr, UnaryExpr,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    CommonJsFullRequireDependency, CommonJsRequireContextDependency, CommonJsRequireDependency,
    RequireHeaderDependency, RequireResolveContextDependency, RequireResolveDependency,
    RequireResolveHeaderDependency, local_module_dependency::LocalModuleDependency,
  },
  magic_comment::try_extract_magic_comment,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    JavascriptParser, context_reg_exp, create_context_dependency, create_traceable_error, expr_name,
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
    phase: None,
  };
  let mut dep = CommonJsRequireContextDependency::new(
    options,
    DependencyRange::from(span)
      .to_loc(Some(parser.source()))
      .expect("Should get correct loc"),
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
    phase: None,
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
  fn has_ignore_comment(parser: &mut JavascriptParser, error_span: Span, span: Span) -> bool {
    if !parser
      .javascript_options
      .commonjs_magic_comments
      .unwrap_or(false)
    {
      return false;
    }

    try_extract_magic_comment(parser, error_span, span)
      .get_ignore()
      .unwrap_or_default()
  }

  fn should_process_resolve(parser: &mut JavascriptParser, call_expr: &CallExpr) -> bool {
    let Callee::Expr(expr) = &call_expr.callee else {
      return false;
    };

    let Expr::Member(member_expr) = expr.as_ref() else {
      return false;
    };

    let Expr::Ident(ident) = member_expr.obj.as_ref() else {
      return false;
    };

    if parser.get_variable_info(&ident.sym).is_some() {
      return false;
    }

    true
  }

  fn process_resolve(&self, parser: &mut JavascriptParser, call_expr: &CallExpr, weak: bool) {
    if call_expr.args.len() != 1 {
      return;
    }

    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = &call_expr.args[0]
      && Self::has_ignore_comment(parser, call_expr.span, argument_expr.span())
    {
      return;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    let require_resolve_header_dependency = Box::new(RequireResolveHeaderDependency::new(
      call_expr.callee.span().into(),
      Some(parser.source()),
    ));

    if param.is_conditional() {
      for option in param.options() {
        if !self.process_resolve_item(parser, option, weak) {
          self.process_resolve_context(parser, option, weak);
        }
      }
      parser.add_dependency(require_resolve_header_dependency);
    } else {
      if !self.process_resolve_item(parser, &param, weak) {
        self.process_resolve_context(parser, &param, weak);
      }
      parser.add_dependency(require_resolve_header_dependency);
    }
  }

  fn process_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
    weak: bool,
  ) -> bool {
    if param.is_string() {
      parser.add_dependency(Box::new(RequireResolveDependency::new(
        param.string().clone(),
        param.range().into(),
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
    let dep = create_require_resolve_context_dependency(parser, param, param.range().into(), weak);

    parser.add_dependency(Box::new(dep));
  }

  fn chain_handler(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    call_expr: &CallExpr,
    members: &[Atom],
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    if call_expr.args.len() != 1 {
      return None;
    }
    let arg = &call_expr.args[0];
    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = arg
      && Self::has_ignore_comment(parser, call_expr.span, argument_expr.span())
    {
      return None;
    }
    let param = parser.evaluate_expression(&arg.expr);
    param.is_string().then(|| {
      CommonJsFullRequireDependency::new(
        param.string().to_owned(),
        members.to_vec(),
        member_expr.span().into(),
        is_call,
        parser.in_try,
        !parser.is_asi_position(member_expr.span_lo()),
        Some(parser.source()),
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
        param.string().clone(),
        range_expr,
        Some(span.into()),
        parser.in_try,
        Some(parser.source()),
      );
      parser.add_dependency(Box::new(dep));
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
    parser.add_dependency(Box::new(dep));
    Some(true)
  }

  fn require_handler(&self, parser: &mut JavascriptParser, expr: CallOrNewExpr) -> Option<bool> {
    let callee = expr.callee()?;
    let args = expr.args()?;

    if args.len() != 1 {
      return None;
    }

    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = &args[0]
      && Self::has_ignore_comment(parser, expr.span(), argument_expr.span())
    {
      return Some(true);
    }

    let param = parser.evaluate_expression(&args[0].expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self.process_require_item(parser, expr.span(), p).is_none() {
          is_expression = true;
        }
      }
      if !is_expression {
        let range: DependencyRange = callee.span().into();
        parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(
          range,
          Some(parser.source()),
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
      parser.add_presentational_dependency(dep);
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
      parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(
        range,
        Some(parser.source()),
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
        phase: None,
      },
      DependencyRange::from(span)
        .to_loc(Some(parser.source()))
        .expect("Should get correct loc"),
      ident.span().into(),
      None,
      parser.in_try,
    );
    if let Some(true) = parser.javascript_options.unknown_context_critical {
      let mut error = create_traceable_error(
        "Critical dependency".into(),
        "require function is used in a way in which dependencies cannot be statically extracted"
          .to_string(),
        parser.source.to_string(),
        ident.span().into(),
      );
      error.severity = Severity::Warning;
      *dep.critical_mut() = Some(Diagnostic::from(error));
    }
    parser.add_dependency(Box::new(dep));
    Some(true)
  }
}

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn can_rename(&self, parser: &mut JavascriptParser, for_name: &str) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      Some(parser.javascript_options.require_alias.unwrap_or(true))
    } else {
      None
    }
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span().into(),
        "undefined".into(),
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
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    match for_name {
      expr_name::REQUIRE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE.into(),
        expr_name::REQUIRE.into(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE.into(),
        expr_name::REQUIRE_RESOLVE.into(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE_WEAK => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE_WEAK.into(),
        expr_name::REQUIRE_RESOLVE_WEAK.into(),
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span().into(),
        "'function'".into(),
      )));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE {
      self.require_handler(parser, CallOrNewExpr::Call(call_expr))
    } else if for_name == expr_name::REQUIRE_RESOLVE {
      if matches!(parser.javascript_options.require_resolve, Some(false))
        || !Self::should_process_resolve(parser, call_expr)
      {
        return None;
      }

      self.process_resolve(parser, call_expr, false);
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE_WEAK {
      if !Self::should_process_resolve(parser, call_expr) {
        return None;
      }

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
    if for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE {
      self.require_handler(parser, CallOrNewExpr::New(new_expr))
    } else {
      None
    }
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    _callee_members: &[Atom],
    call_expr: &CallExpr,
    members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && let Some(dep) = self.chain_handler(parser, member_expr, call_expr, members, false)
    {
      parser.add_dependency(Box::new(dep));
      return Some(true);
    }
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    _callee_members: &[Atom],
    inner_call_expr: &CallExpr,
    members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && let Some(callee) = call_expr.callee.as_expr()
      && let Some(member) = callee.as_member()
      && let Some(dep) = self.chain_handler(parser, member, inner_call_expr, members, true)
    {
      parser.add_dependency(Box::new(dep));
      parser.walk_expr_or_spread(&call_expr.args);
      return Some(true);
    }
    None
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
    _expr: &AssignExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (0, 0).into(),
        "var require;".into(),
      )));
      return Some(true);
    }

    None
  }
}
