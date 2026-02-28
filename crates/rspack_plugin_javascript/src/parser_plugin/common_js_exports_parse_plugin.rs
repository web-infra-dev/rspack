use rspack_core::{BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals};
use rspack_util::SpanExt;
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{
    AssignExpr, CallExpr, Expr, ExprOrSpread, Ident, Lit, MemberExpr, ObjectLit, Prop, PropName,
    PropOrSpread, UnaryExpr, UnaryOp,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsSelfReferenceDependency,
    ExportsBase, ModuleDecoratorDependency,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::JavascriptParser,
};

fn get_value_of_property_description(expr: &Expr) -> Option<&Expr> {
  if let Expr::Object(ObjectLit { props, .. }) = expr {
    for prop in props {
      if let PropOrSpread::Prop(prop) = prop
        && let Prop::KeyValue(key_value_prop) = &**prop
        && let PropName::Ident(ident) = &key_value_prop.key
        && &ident.sym == "value"
      {
        return Some(&key_value_prop.value);
      }
    }
  }
  None
}

fn is_truthy_literal(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(lit) => is_lit_truthy_literal(lit),
    Expr::Unary(unary) => {
      if unary.op == UnaryOp::Bang {
        return is_falsy_literal(&unary.arg);
      }
      false
    }
    _ => false,
  }
}

fn is_falsy_literal(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(lit) => !is_lit_truthy_literal(lit),
    Expr::Unary(unary) => {
      if unary.op == UnaryOp::Bang {
        return is_truthy_literal(&unary.arg);
      }
      false
    }
    _ => false,
  }
}

fn is_lit_truthy_literal(lit: &Lit) -> bool {
  match lit {
    Lit::Str(str) => !str.value.is_empty(),
    Lit::Bool(bool) => bool.value,
    Lit::Null(_) => false,
    Lit::Num(num) => num.value != 0.0,
    _ => true,
  }
}

impl JavascriptParser<'_> {
  // can't scan `__esModule` value
  fn bailout(&mut self) {
    if matches!(self.parser_exports_state, Some(true)) {
      self.build_meta.exports_type = BuildMetaExportsType::Unset;
      self.build_meta.default_object = BuildMetaDefaultObject::False;
    }
    self.parser_exports_state = Some(false);
  }

  // `__esModule` is false
  fn enable(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) {
      return;
    }
    if self.parser_exports_state.is_none() {
      self.build_meta.exports_type = BuildMetaExportsType::Default;
      self.build_meta.default_object = BuildMetaDefaultObject::Redirect;
    }
    self.parser_exports_state = Some(true);
  }

  // `__esModule` is true
  fn set_flagged(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if matches!(self.build_meta.exports_type, BuildMetaExportsType::Dynamic) {
      return;
    }
    self.build_meta.exports_type = BuildMetaExportsType::Flagged;
  }

  // `__esModule` is dynamic, eg `true && true`
  fn set_dynamic(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    self.build_meta.exports_type = BuildMetaExportsType::Dynamic;
  }

  fn check_namespace(&mut self, top_level: bool, value_expr: Option<&Expr>) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if let Some(value_expr) = value_expr
      && is_truthy_literal(value_expr)
      && top_level
    {
      self.set_flagged();
    } else {
      self.set_dynamic();
    }
  }
}

fn parse_require_call<'a>(
  parser: &mut JavascriptParser,
  mut expr: &'a Expr,
) -> Option<(BasicEvaluatedExpression<'a>, Vec<Atom>)> {
  let mut ids = Vec::new();
  while let Some(member) = expr.as_member() {
    if let Some(prop) = member.prop.as_ident() {
      ids.push(prop.sym.clone());
    } else if let Some(prop) = member.prop.as_computed()
      && let prop = parser.evaluate_expression(&prop.expr)
      && let Some(prop) = prop.as_string()
    {
      ids.push(prop.into());
    } else {
      return None;
    }
    expr = &*member.obj;
  }
  if let Some(call) = expr.as_call()
    && call.args.len() == 1
    && let Some(callee) = call.callee.as_expr()
    && let Some(callee) = callee.as_ident()
    && let Some(info) = parser.get_free_info_from_variable(&callee.sym)
    && info.name == "require"
  {
    let arg = &call.args[0];
    if arg.spread.is_some() {
      return None;
    }
    let arg = parser.evaluate_expression(&arg.expr);
    ids.reverse();
    return Some((arg, ids));
  }
  None
}

fn handle_assign_export(
  parser: &mut JavascriptParser,
  assign_expr: &AssignExpr,
  remaining: &[Atom],
  base: ExportsBase,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if (remaining.is_empty() || remaining.first().is_some_and(|i| i != "__esModule"))
    && let Some((arg, ids)) = parse_require_call(parser, &assign_expr.right)
    && arg.is_string()
  {
    parser.enable();
    if remaining.is_empty() {
      // exports = require('xx');
      // module.exports = require('xx');
      // this = require('xx');
      // It's possible to reexport __esModule, so we must convert to a dynamic module
      parser.set_dynamic();
    }
    // exports.aaa = require('xx');
    // module.exports.aaa = require('xx');
    // this.aaa = require('xx');
    let range: DependencyRange = assign_expr.span.into();
    parser.add_dependency(Box::new(CommonJsExportRequireDependency::new(
      arg.string().clone(),
      parser.in_try,
      range,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_expr.span()),
    )));
    return Some(true);
  }

  if remaining.is_empty() {
    return None;
  }

  parser.enable();
  // exports.__esModule = true;
  // module.exports.__esModule = true;
  // this.__esModule = true;
  if let Some(first_member) = remaining.first()
    && first_member == "__esModule"
  {
    parser.check_namespace(
      // const flagIt = () => (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = false
      // const flagIt = () => { exports.__esModule = true }; => stmt_level = 2, last_stmt_is_expr_stmt = true
      // (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = true
      parser.statement_path.len() == 1 && parser.is_statement_level_expression(assign_expr.span()),
      Some(&assign_expr.right),
    );
  }
  // exports.a = 1;
  // module.exports.a = 1;
  // this.a = 1;
  parser.add_dependency(Box::new(CommonJsExportsDependency::new(
    assign_expr.left.span().into(),
    None,
    base,
    remaining.to_owned(),
  )));
  parser.walk_expression(&assign_expr.right);
  Some(true)
}

fn handle_access_export(
  parser: &mut JavascriptParser,
  expr_span: Span,
  remaining: &[Atom],
  remaining_optionals: &[bool],
  base: ExportsBase,
  call_args: Option<&Vec<ExprOrSpread>>,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if remaining.is_empty() {
    parser.bailout();
  }
  parser.add_dependency(Box::new(CommonJsSelfReferenceDependency::new(
    expr_span.into(),
    base,
    remaining.to_vec(),
    remaining_optionals.to_vec(),
    true,
  )));
  if let Some(call_args) = call_args {
    parser.walk_expr_or_spread(call_args);
  }
  Some(true)
}

pub struct CommonJsExportsParserPlugin {
  skip_in_esm: bool,
}

impl CommonJsExportsParserPlugin {
  pub fn new(skip_in_esm: bool) -> Self {
    Self { skip_in_esm }
  }

  fn should_skip_handler(&self, parser: &JavascriptParser) -> bool {
    self.skip_in_esm && parser.is_esm
  }
}

impl JavascriptParserPlugin for CommonJsExportsParserPlugin {
  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    assign_expr: &AssignExpr,
    remaining: &[Atom],
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.x = y;
      return handle_assign_export(parser, assign_expr, remaining, ExportsBase::Exports);
    }
    if for_name == "module" && matches!(remaining.first(), Some(first) if first == "exports") {
      // module.exports.x = y;
      return handle_assign_export(
        parser,
        assign_expr,
        &remaining[1..],
        ExportsBase::ModuleExports,
      );
    }
    if for_name == "this" && parser.is_top_level_scope() {
      // this.x = y
      return handle_assign_export(parser, assign_expr, remaining, ExportsBase::This);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_esm {
      return None;
    }
    if for_name == "Object.defineProperty"
      && parser.is_statement_level_expression(call_expr.span())
      && call_expr.args.len() == 3
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg0,
      }) = call_expr.args.first()
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg1,
      }) = call_expr.args.get(1)
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg2,
      }) = call_expr.args.get(2)
    {
      let exports_arg = parser.evaluate_expression(arg0);
      if !exports_arg.is_identifier() {
        return None;
      }
      let base = match exports_arg.identifier().as_str() {
        "exports" => ExportsBase::DefinePropertyExports,
        "module.exports" => ExportsBase::DefinePropertyModuleExports,
        "this" if parser.is_top_level_scope() => ExportsBase::DefinePropertyThis,
        _ => return None,
      };
      let property = parser.evaluate_expression(arg1).as_string()?;
      parser.enable();
      // Object.defineProperty(exports, "__esModule", { value: true });
      // Object.defineProperty(module.exports, "__esModule", { value: true });
      // Object.defineProperty(this, "__esModule", { value: true });
      if &property == "__esModule" {
        parser.check_namespace(
          parser.statement_path.len() == 1,
          get_value_of_property_description(arg2),
        );
      }
      parser.add_dependency(Box::new(CommonJsExportsDependency::new(
        call_expr.span.into(),
        Some(arg2.span().into()),
        base,
        vec![property.into()],
      )));

      parser.walk_expression(arg2);
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
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module" {
      let decorator = if parser.is_esm {
        RuntimeGlobals::ESM_MODULE_DECORATOR
      } else {
        RuntimeGlobals::NODE_MODULE_DECORATOR
      };
      parser.bailout();
      parser.add_dependency(Box::new(ModuleDecoratorDependency::new(
        decorator,
        !parser.is_esm,
      )));
      return Some(true);
    }

    if for_name == "exports" {
      // exports
      return handle_access_export(parser, ident.span(), &[], &[], ExportsBase::Exports, None);
    }

    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_top_level_this() {
      // this
      return handle_access_export(parser, expr.span(), &[], &[], ExportsBase::This, None);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module.exports" {
      // module.exports
      return handle_access_export(
        parser,
        expr.span(),
        &[],
        &[],
        ExportsBase::ModuleExports,
        None,
      );
    }
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c
      return handle_access_export(
        parser,
        expr.span(),
        members,
        members_optionals,
        ExportsBase::Exports,
        None,
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c
      return handle_access_export(
        parser,
        expr.span(),
        &members[1..],
        &members_optionals[1..],
        ExportsBase::ModuleExports,
        None,
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c
      return handle_access_export(
        parser,
        expr.span(),
        members,
        members_optionals,
        ExportsBase::This,
        None,
      );
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        members,
        members_optionals,
        ExportsBase::Exports,
        Some(&expr.args),
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        &members[1..],
        &members_optionals[1..],
        ExportsBase::ModuleExports,
        Some(&expr.args),
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        members,
        members_optionals,
        ExportsBase::This,
        Some(&expr.args),
      );
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if self.should_skip_handler(parser) {
      return None;
    }

    (for_name == "module" || for_name == "exports").then(|| {
      eval::evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      )
    })
  }
}
