use rspack_core::{
  BoxDependency, BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals,
};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  Argument, AssignmentExpression, Ast, CallExpression, Expr, ExprData, GetSpan, PropertyKeyData,
  Span, ThisExpression, TypedSubRange, UnaryExpression, UnaryOperator,
};

use super::JavascriptParserPlugin;
use crate::{
  Atom,
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsSelfReferenceDependency,
    ExportsBase, ModuleDecoratorDependency,
  },
  parser_plugin::common_js_imports_parse_plugin::is_require_call_expr,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::get_value_by_obj_prop,
  },
  visitors::{HookMemberExpression, Identifier, JavascriptParser, member_property_key_to_atom},
};

fn get_value_of_property_description(ast: &Ast<'_>, expr: Expr) -> Option<Expr> {
  get_value_by_obj_prop(ast, expr.as_object_expression(ast)?, "value")
}

fn is_truthy_literal(ast: &Ast<'_>, expr: Expr) -> bool {
  match ast.expr_data(expr) {
    ExprData::StringLiteral(string) => !ast.get_wtf8(string.value(ast)).is_empty(),
    ExprData::BooleanLiteral(boolean) => boolean.value(ast),
    ExprData::NullLiteral(_) => false,
    ExprData::NumericLiteral(number) => number.value(ast) != 0.0,
    ExprData::BigIntLiteral(_) | ExprData::RegExpLiteral(_) => true,
    ExprData::UnaryExpression(unary) => {
      if unary.operator(ast) == UnaryOperator::LogicalNot {
        return is_falsy_literal(ast, unary.argument(ast));
      }
      false
    }
    _ => false,
  }
}

fn is_falsy_literal(ast: &Ast<'_>, expr: Expr) -> bool {
  match ast.expr_data(expr) {
    ExprData::StringLiteral(_)
    | ExprData::BooleanLiteral(_)
    | ExprData::NullLiteral(_)
    | ExprData::NumericLiteral(_)
    | ExprData::BigIntLiteral(_)
    | ExprData::RegExpLiteral(_) => !is_truthy_literal(ast, expr),
    ExprData::UnaryExpression(unary) => {
      if unary.operator(ast) == UnaryOperator::LogicalNot {
        return is_truthy_literal(ast, unary.argument(ast));
      }
      false
    }
    _ => false,
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

  fn check_namespace(&mut self, top_level: bool, value_expr: Option<Expr>) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if let Some(value_expr) = value_expr
      && is_truthy_literal(self.ast.ast, value_expr)
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
  mut expr: Expr,
) -> Option<(BasicEvaluatedExpression<'p>, Vec<Atom>)> {
  let mut ids = Vec::new();
  while let Some(member) = expr.as_member_expression(parser.ast.ast) {
    let ast = parser.ast.ast;
    match ast.property_key_data(member.property(ast)) {
      PropertyKeyData::IdentifierName(property) if !member.computed(ast) => {
        ids.push(Atom::from(ast.get_utf8(property.name(ast))));
      }
      _ if member.computed(ast) => {
        ids.push(member_property_key_to_atom(ast, member.property(ast))?);
      }
      _ => return None,
    }
    expr = member.object(parser.ast.ast);
  }
  if let Some(call) = expr.as_call_expression(parser.ast.ast)
    && is_require_call_expr(parser, call)
  {
    let ast = parser.ast.ast;
    let arg = call.arguments(ast).get_node(ast, 0)?.as_expr(ast)?;
    let arg = parser.evaluate_expression(arg);
    ids.reverse();
    return Some((arg, ids));
  }
  None
}

fn handle_assign_export(
  parser: &mut JavascriptParser,
  assign_expr: AssignmentExpression,
  remaining: &[Atom],
  base: ExportsBase,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  let assign_span = assign_expr.span(parser.ast.ast);
  if (remaining.is_empty() || remaining.first().is_some_and(|i| i != "__esModule"))
    && let Some((arg, ids)) = parse_require_call(parser, assign_expr.right(parser.ast.ast))
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
    let range: DependencyRange = assign_span.into();
    parser.add_dependency(BoxDependency::new(CommonJsExportRequireDependency::new(
      arg.string().to_owned(),
      parser.in_try,
      range,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_span),
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
      parser.statement_path.len() == 1 && parser.is_statement_level_expression(assign_span),
      Some(assign_expr.right(parser.ast.ast)),
    );
  }
  // exports.a = 1;
  // module.exports.a = 1;
  // this.a = 1;
  parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
    assign_expr.left(parser.ast.ast).span(parser.ast.ast).into(),
    None,
    base,
    remaining.to_owned(),
  )));
  parser.walk_expression(assign_expr.right(parser.ast.ast));
  Some(true)
}

fn handle_access_export(
  parser: &mut JavascriptParser,
  expr_span: Span,
  remaining: &[Atom],
  remaining_optionals: &[bool],
  base: ExportsBase,
  call_args: Option<TypedSubRange<Argument>>,
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
    let ast = parser.ast.ast;
    parser.walk_arguments(call_args.iter().map(|id| ast.get_node_in_sub_range(id)));
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
    assign_expr: AssignmentExpression,
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
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_esm {
      return None;
    }
    let ast = parser.ast.ast;
    let call_span = call_expr.span(ast);
    let args = call_expr.arguments(ast);
    if for_name == "Object.defineProperty"
      && parser.is_statement_level_expression(call_span)
      && args.len() == 3
      && let Some(arg0) = args
        .get_node(ast, 0)
        .and_then(|argument| argument.as_expr(ast))
      && let Some(arg1) = args
        .get_node(ast, 1)
        .and_then(|argument| argument.as_expr(ast))
      && let Some(arg2) = args
        .get_node(ast, 2)
        .and_then(|argument| argument.as_expr(ast))
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
          get_value_of_property_description(parser.ast.ast, arg2),
        );
      }
      parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
        call_span.into(),
        Some(arg2.span(parser.ast.ast).into()),
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
    ident: &Identifier,
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
      return handle_access_export(parser, ident.span(), &[], &[], ExportsBase::Exports, None);
    }

    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: ThisExpression,
    _for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_top_level_this() {
      // this
      return handle_access_export(
        parser,
        expr.span(parser.ast.ast),
        &[],
        &[],
        ExportsBase::This,
        None,
      );
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module.exports" {
      // module.exports
      return handle_access_export(
        parser,
        expr.span(parser.ast.ast),
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
    expr: HookMemberExpression,
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
        expr.span(parser.ast.ast),
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
        expr.span(parser.ast.ast),
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
        expr.span(parser.ast.ast),
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
    expr: CallExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }
    let ast = parser.ast.ast;
    let callee_span = expr.callee(ast).span(ast);
    let arguments = || expr.arguments(ast);

    if for_name == "exports" {
      // exports.a.b.c()
      return handle_access_export(
        parser,
        callee_span,
        members,
        members_optionals,
        ExportsBase::Exports,
        Some(arguments()),
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c()
      return handle_access_export(
        parser,
        callee_span,
        &members[1..],
        &members_optionals[1..],
        ExportsBase::ModuleExports,
        Some(arguments()),
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c()
      return handle_access_export(
        parser,
        callee_span,
        members,
        members_optionals,
        ExportsBase::This,
        Some(arguments()),
      );
    }

    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if self.should_skip_handler(parser) {
      return None;
    }

    let span = expr.span(parser.ast.ast);
    (for_name == "module" || for_name == "exports")
      .then(|| eval::evaluate_to_string("object".to_string(), span.real_lo(), span.real_hi()))
  }
}
