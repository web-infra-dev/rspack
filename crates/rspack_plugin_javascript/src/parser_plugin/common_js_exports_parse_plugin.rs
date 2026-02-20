use rspack_core::{BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals};
use rspack_util::{SpanExt, atom::Atom};
use swc_experimental_ecma_ast::{
  AssignExpr, Ast, CallExpr, Expr, ExprOrSpread, GetSpan, Ident, Lit, MemberExpr, Prop, PropName,
  PropOrSpread, Span, ThisExpr, TypedSubRange, UnaryExpr, UnaryOp,
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

fn get_value_of_property_description(ast: &Ast, expr: Expr) -> Option<Expr> {
  if let Expr::Object(obj_lit) = expr {
    for prop in obj_lit.props(ast).iter() {
      let prop = ast.get_node_in_sub_range(prop);
      if let PropOrSpread::Prop(prop) = prop
        && let Prop::KeyValue(key_value_prop) = prop
        && let PropName::Ident(ident) = key_value_prop.key(ast)
        && ast.get_utf8(ident.sym(ast)) == "value"
      {
        return Some(key_value_prop.value(ast));
      }
    }
  }
  None
}

fn is_truthy_literal(ast: &Ast, expr: Expr) -> bool {
  match expr {
    Expr::Lit(lit) => is_lit_truthy_literal(ast, lit),
    Expr::Unary(unary) => {
      if unary.op(ast) == UnaryOp::Bang {
        return is_falsy_literal(ast, unary.arg(ast));
      }
      false
    }
    _ => false,
  }
}

fn is_falsy_literal(ast: &Ast, expr: Expr) -> bool {
  match expr {
    Expr::Lit(lit) => !is_lit_truthy_literal(ast, lit),
    Expr::Unary(unary) => {
      if unary.op(ast) == UnaryOp::Bang {
        return is_truthy_literal(ast, unary.arg(ast));
      }
      false
    }
    _ => false,
  }
}

fn is_lit_truthy_literal(ast: &Ast, lit: Lit) -> bool {
  match lit {
    Lit::Str(str) => !ast.get_wtf8(str.value(ast)).is_empty(),
    Lit::Bool(bool) => bool.value(ast),
    Lit::Null(_) => false,
    Lit::Num(num) => num.value(ast) != 0.0,
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

  fn check_namespace(&mut self, top_level: bool, value_expr: Option<Expr>) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if let Some(value_expr) = value_expr
      && is_truthy_literal(&self.ast, value_expr)
      && top_level
    {
      self.set_flagged();
    } else {
      self.set_dynamic();
    }
  }
}

fn parse_require_call(
  parser: &mut JavascriptParser,
  mut expr: Expr,
) -> Option<(BasicEvaluatedExpression, Vec<Atom>)> {
  let mut ids = Vec::new();
  while let Some(member) = expr.as_member() {
    if let Some(prop) = member.prop(&parser.ast).as_ident() {
      ids.push(parser.ast.get_atom(prop.sym(&parser.ast)));
    } else if let Some(prop) = member.prop(&parser.ast).as_computed()
      && let prop = parser.evaluate_expression(prop.expr(&parser.ast))
      && let Some(prop) = prop.as_string()
    {
      ids.push(prop.into());
    } else {
      return None;
    }
    expr = member.obj(&parser.ast);
  }
  if let Some(call) = expr.as_call()
    && call.args(&parser.ast).len() == 1
    && let Some(callee) = call.callee(&parser.ast).as_expr()
    && let Some(callee) = callee.as_ident()
    && let Some(info) =
      parser.get_free_info_from_variable(&parser.ast.get_atom(callee.sym(&parser.ast)))
    && info.name == "require"
  {
    let arg = call.args(&parser.ast).get_node(&parser.ast, 0).unwrap();
    if arg.spread(&parser.ast).is_some() {
      return None;
    }
    let arg = parser.evaluate_expression(arg.expr(&parser.ast));
    ids.reverse();
    return Some((arg, ids));
  }
  None
}

fn handle_assign_export(
  parser: &mut JavascriptParser,
  assign_expr: AssignExpr,
  remaining: &[Atom],
  base: ExportsBase,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if (remaining.is_empty() || remaining.first().is_some_and(|i| i != "__esModule"))
    && let Some((arg, ids)) = parse_require_call(parser, assign_expr.right(&parser.ast))
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
    let range: DependencyRange = assign_expr.span(&parser.ast).into();
    parser.add_dependency(Box::new(CommonJsExportRequireDependency::new(
      arg.string().clone(),
      parser.in_try,
      range,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_expr.span(&parser.ast)),
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
      parser.statement_path.len() == 1
        && parser.is_statement_level_expression(assign_expr.span(&parser.ast)),
      Some(assign_expr.right(&parser.ast)),
    );
  }
  // exports.a = 1;
  // module.exports.a = 1;
  // this.a = 1;
  parser.add_dependency(Box::new(CommonJsExportsDependency::new(
    assign_expr.left(&parser.ast).span(&parser.ast).into(),
    None,
    base,
    remaining.to_owned(),
  )));
  parser.walk_expression(assign_expr.right(&parser.ast));
  Some(true)
}

fn handle_access_export(
  parser: &mut JavascriptParser,
  expr_span: Span,
  remaining: &[Atom],
  base: ExportsBase,
  call_args: TypedSubRange<ExprOrSpread>,
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
    true,
  )));
  parser.walk_expr_or_spread(call_args);
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
    assign_expr: AssignExpr,
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
    call_expr: CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_esm {
      return None;
    }

    let arg0 = call_expr
      .args(&parser.ast)
      .get_node(&parser.ast, 0)
      .unwrap();
    if arg0.spread(&parser.ast).is_some() {
      return None;
    }
    let arg0 = arg0.expr(&parser.ast);

    let arg1 = call_expr
      .args(&parser.ast)
      .get_node(&parser.ast, 1)
      .unwrap();
    if arg1.spread(&parser.ast).is_some() {
      return None;
    }
    let arg1 = arg1.expr(&parser.ast);

    let arg2 = call_expr
      .args(&parser.ast)
      .get_node(&parser.ast, 2)
      .unwrap();
    if arg2.spread(&parser.ast).is_some() {
      return None;
    }
    let arg2 = arg2.expr(&parser.ast);

    if for_name == "Object.defineProperty"
      && parser.is_statement_level_expression(call_expr.span(&parser.ast))
      && call_expr.args(&parser.ast).len() == 3
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
          get_value_of_property_description(&parser.ast, arg2),
        );
      }
      parser.add_dependency(Box::new(CommonJsExportsDependency::new(
        call_expr.span(&parser.ast).into(),
        Some(arg2.span(&parser.ast).into()),
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
    ident: Ident,
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
      return handle_access_export(
        parser,
        ident.span(&parser.ast),
        &[],
        ExportsBase::Exports,
        TypedSubRange::empty(),
      );
    }

    None
  }

  fn this(&self, parser: &mut JavascriptParser, expr: ThisExpr, _for_name: &str) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_top_level_this() {
      // this
      return handle_access_export(
        parser,
        expr.span(&parser.ast),
        &[],
        ExportsBase::This,
        TypedSubRange::empty(),
      );
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module.exports" {
      // module.exports
      return handle_access_export(
        parser,
        expr.span(&parser.ast),
        &[],
        ExportsBase::ModuleExports,
        TypedSubRange::empty(),
      );
    }
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: MemberExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c
      return handle_access_export(
        parser,
        expr.span(&parser.ast),
        members,
        ExportsBase::Exports,
        TypedSubRange::empty(),
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c
      return handle_access_export(
        parser,
        expr.span(&parser.ast),
        &members[1..],
        ExportsBase::ModuleExports,
        TypedSubRange::empty(),
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c
      return handle_access_export(
        parser,
        expr.span(&parser.ast),
        members,
        ExportsBase::This,
        TypedSubRange::empty(),
      );
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: CallExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee(&parser.ast).span(&parser.ast),
        members,
        ExportsBase::Exports,
        expr.args(&parser.ast),
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee(&parser.ast).span(&parser.ast),
        &members[1..],
        ExportsBase::ModuleExports,
        expr.args(&parser.ast),
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c()
      return handle_access_export(
        parser,
        expr.callee(&parser.ast).span(&parser.ast),
        members,
        ExportsBase::This,
        expr.args(&parser.ast),
      );
    }

    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    if self.should_skip_handler(parser) {
      return None;
    }

    (for_name == "module" || for_name == "exports").then(|| {
      eval::evaluate_to_string(
        "object".to_string(),
        expr.span(&parser.ast).real_lo(),
        expr.span(&parser.ast).real_hi(),
      )
    })
  }
}
