use std::sync::Arc;

use rspack_core::{
  BoxDependency, BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals,
  RuntimeRequirementsDependency,
};
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{
  AssignExpr, CallExpr, Expr, ExprOrSpread, GetSpan, Ident, Lit, MemberExpr, Prop, PropName,
  PropOrSpread, Span, ThisExpr, UnaryExpr, UnaryOp,
};

use super::JavascriptParserPlugin;
use crate::{
  Atom,
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsObjectExportDependency,
    CommonJsObjectExportKind, CommonJsSelfReferenceDependency, ExportsBase,
    ModuleDecoratorDependency,
  },
  parser_plugin::common_js_imports_parse_plugin::is_require_call_expr,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::JavascriptParser,
};

fn get_value_of_property_description<'a>(expr: &'a Expr<'a>) -> Option<&'a Expr<'a>> {
  if let Expr::Object(obj) = expr {
    for prop in &obj.props {
      if let PropOrSpread::Prop(prop) = prop
        && let Prop::KeyValue(key_value_prop) = &**prop
        && let PropName::Ident(ident) = &key_value_prop.key
        && ident.sym == "value"
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
    Lit::Str(str) => !str.value.as_wtf8().is_empty(),
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
      self.build_meta.clear_exports_type();
      self
        .build_meta
        .set_default_object(BuildMetaDefaultObject::False);
    }
    self.parser_exports_state = Some(false);
  }

  // `__esModule` is false
  fn enable(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) {
      return;
    }
    if self.parser_exports_state.is_none() {
      self
        .build_meta
        .set_exports_type(BuildMetaExportsType::Default);
      self
        .build_meta
        .set_default_object(BuildMetaDefaultObject::Redirect);
    }
    self.parser_exports_state = Some(true);
  }

  // `__esModule` is true
  fn set_flagged(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if matches!(
      self.build_meta.exports_type(),
      BuildMetaExportsType::Dynamic
    ) {
      return;
    }
    self
      .build_meta
      .set_exports_type(BuildMetaExportsType::Flagged);
  }

  // `__esModule` is dynamic, eg `true && true`
  fn set_dynamic(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    self
      .build_meta
      .set_exports_type(BuildMetaExportsType::Dynamic);
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

fn parse_require_call<'p: 'a, 'a>(
  parser: &mut JavascriptParser<'p>,
  mut expr: &'a Expr<'a>,
) -> Option<(BasicEvaluatedExpression<'a>, Vec<Atom>)> {
  let mut ids = Vec::new();
  while let Some(member) = expr.as_member() {
    if let Some(prop) = member.prop.as_ident() {
      ids.push(Atom::from(&prop.sym));
    } else if let Some(prop) = member.prop.as_computed()
      && let prop = parser.evaluate_expression(&prop.expr)
      && let Some(prop) = prop.as_string()
    {
      ids.push(prop.into());
    } else {
      return None;
    }
    expr = &member.obj;
  }
  if let Some(call) = expr.as_call()
    && is_require_call_expr(parser, call)
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

fn get_static_property_name(name: &PropName) -> Option<Atom> {
  let name = match name {
    PropName::Ident(ident) => Atom::from(&ident.sym),
    PropName::Str(str) => Atom::from(str.value.to_string_lossy().as_ref()),
    PropName::Num(_) | PropName::Computed(_) | PropName::BigInt(_) => return None,
  };
  (name != "__proto__").then_some(name)
}

fn get_object_export_name(prop: &PropOrSpread) -> Option<Atom> {
  let PropOrSpread::Prop(prop) = prop else {
    return None;
  };
  match &**prop {
    Prop::Shorthand(ident) => {
      let name = Atom::from(&ident.sym);
      (name != "__proto__").then_some(name)
    }
    Prop::KeyValue(prop) => get_static_property_name(&prop.key),
    Prop::Getter(prop) => get_static_property_name(&prop.key),
    Prop::Setter(prop) => get_static_property_name(&prop.key),
    Prop::Method(prop) => get_static_property_name(&prop.key),
    Prop::Assign(_) => None,
  }
}

fn get_object_export_kind(prop: &Prop) -> Option<CommonJsObjectExportKind> {
  Some(match prop {
    Prop::KeyValue(_) => CommonJsObjectExportKind::KeyValue,
    Prop::Shorthand(_) => CommonJsObjectExportKind::Shorthand,
    Prop::Getter(_) => CommonJsObjectExportKind::Getter,
    Prop::Setter(_) => CommonJsObjectExportKind::Setter,
    Prop::Method(prop) => match (prop.function.is_async, prop.function.is_generator) {
      (false, false) => CommonJsObjectExportKind::Method,
      (true, false) => CommonJsObjectExportKind::AsyncMethod,
      (false, true) => CommonJsObjectExportKind::GeneratorMethod,
      (true, true) => CommonJsObjectExportKind::AsyncGeneratorMethod,
    },
    Prop::Assign(_) => return None,
  })
}

fn expand_parenthesized_range(source: &str, mut range: DependencyRange) -> DependencyRange {
  let bytes = source.as_bytes();
  loop {
    let mut start = range.start as usize;
    while start > 0 && bytes[start - 1].is_ascii_whitespace() {
      start -= 1;
    }
    let mut end = range.end as usize;
    while end < bytes.len() && bytes[end].is_ascii_whitespace() {
      end += 1;
    }
    if start == 0 || end >= bytes.len() || bytes[start - 1] != b'(' || bytes[end] != b')' {
      break;
    }
    range.start = (start - 1) as u32;
    range.end = (end + 1) as u32;
  }
  range
}

fn get_object_export_ranges(
  prop: &Prop,
  source: &str,
) -> Option<(DependencyRange, DependencyRange, DependencyRange)> {
  let mut range: DependencyRange = prop.span().into();
  let key_range = match prop {
    Prop::Shorthand(ident) => ident.span.into(),
    Prop::KeyValue(prop) => prop.key.span().into(),
    Prop::Getter(prop) => prop.key.span().into(),
    Prop::Setter(prop) => prop.key.span().into(),
    Prop::Method(prop) => prop.key.span().into(),
    Prop::Assign(_) => return None,
  };
  let value_range = match prop {
    Prop::Shorthand(ident) => ident.span.into(),
    // Parentheses are removed before dependency scanning, while the source
    // text is kept. Recover them so source replacements don't leave behind an
    // unmatched closing parenthesis.
    Prop::KeyValue(prop) => expand_parenthesized_range(source, prop.value.span().into()),
    Prop::Getter(prop) => DependencyRange::new(prop.key.span().real_hi(), prop.span.real_hi()),
    Prop::Setter(prop) => DependencyRange::new(prop.key.span().real_hi(), prop.span.real_hi()),
    Prop::Method(prop) => DependencyRange::new(prop.key.span().real_hi(), prop.span().real_hi()),
    Prop::Assign(_) => return None,
  };
  range.end = range.end.max(value_range.end);
  Some((range, key_range, value_range))
}

fn get_object_export_value<'a>(prop: &'a Prop<'a>) -> Option<&'a Expr<'a>> {
  match prop {
    Prop::KeyValue(prop) => Some(&prop.value),
    Prop::Shorthand(_) | Prop::Assign(_) | Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => {
      None
    }
  }
}

fn handle_object_literal_export(
  parser: &mut JavascriptParser,
  assign_expr: &AssignExpr,
  base: ExportsBase,
) -> Option<bool> {
  if !matches!(base, ExportsBase::ModuleExports)
    || parser.statement_path.len() != 1
    || !parser.is_statement_level_expression(assign_expr.span)
  {
    return None;
  }
  let Expr::Object(object) = &assign_expr.right else {
    return None;
  };

  // The original `module.exports` text remains in place, so make sure the
  // generated module wrapper keeps its module argument.
  parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::add_only(
    RuntimeGlobals::MODULE,
  )));

  // A single unknown key makes the object opaque. Resolve all names before
  // adding any structured export dependencies.
  let Some(names) = object
    .props
    .iter()
    .map(get_object_export_name)
    .collect::<Option<Vec<_>>>()
  else {
    parser.bailout();
    parser.walk_expression(&assign_expr.right);
    return Some(true);
  };

  parser.enable();
  for (prop, name) in object.props.iter().zip(names) {
    let PropOrSpread::Prop(prop) = prop else {
      unreachable!("spread properties were rejected while collecting export names")
    };
    let kind = get_object_export_kind(prop)
      .expect("unsupported properties were rejected while collecting export names");
    let (range, key_range, value_range) = get_object_export_ranges(prop, parser.source)
      .expect("unsupported properties were rejected while collecting export names");

    if name == "__esModule" {
      parser.check_namespace(true, get_object_export_value(prop));
    }

    parser.add_dependency(BoxDependency::new(CommonJsObjectExportDependency::new(
      range,
      key_range,
      value_range,
      name,
      kind,
    )));
    parser.walk_property(prop);
  }
  Some(true)
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
    parser.add_dependency(BoxDependency::new(CommonJsExportRequireDependency::new(
      arg.string().clone(),
      parser.in_try,
      range,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_expr.span),
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
      parser.statement_path.len() == 1 && parser.is_statement_level_expression(assign_expr.span),
      Some(&assign_expr.right),
    );
  }
  // exports.a = 1;
  // module.exports.a = 1;
  // this.a = 1;
  parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
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
  call_args: Option<&[ExprOrSpread<'_>]>,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if remaining.is_empty() {
    parser.bailout();
  }
  parser.add_dependency(BoxDependency::new(CommonJsSelfReferenceDependency::new(
    expr_span.into(),
    base,
    remaining.to_vec(),
    remaining_optionals.to_vec(),
    call_args.is_some(),
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

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CommonJsExportsParserPlugin {
  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    assign_expr: &AssignExpr,
    remaining: &[Atom],
    _member_ranges: &[Span],
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
      if let Some(result) =
        handle_object_literal_export(parser, assign_expr, ExportsBase::ModuleExports)
      {
        return Some(result);
      }
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
    parser: &mut JavascriptParser<'p>,
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
      && parser.is_statement_level_expression(call_expr.span)
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
      parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
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
    parser: &mut JavascriptParser<'p>,
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
      parser.add_dependency(BoxDependency::new(ModuleDecoratorDependency::new(
        decorator,
        !parser.is_esm,
      )));
      return Some(true);
    }

    if for_name == "exports" {
      // exports
      return handle_access_export(parser, ident.span, &[], &[], ExportsBase::Exports, None);
    }

    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_top_level_this() {
      // this
      return handle_access_export(parser, expr.span, &[], &[], ExportsBase::This, None);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
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
        expr.span,
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
    parser: &mut JavascriptParser<'p>,
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
        expr.span,
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
        expr.span,
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
        expr.span,
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
    parser: &mut JavascriptParser<'p>,
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

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
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
