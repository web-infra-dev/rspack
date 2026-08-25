use std::sync::Arc;

use rspack_core::{
  ConstDependency, JavascriptParserCommonjsExportsOption, OverrideStrict, RuntimeGlobals,
  RuntimeRequirementsDependency,
};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, CallExpr, Ident, MemberExpr, NewExpr, Program, Span, ThisExpr, UnaryExpr,
};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier},
  visitors::{JavascriptParser, Statement, expr_name},
};

pub struct CommonJsPlugin;

const COMMONJS_REQUIRE_ACCESS_TAG: &str = "commonjs require access";
const COMMONJS_DEFINE_ACCESS_TAG: &str = "commonjs define access";

impl CommonJsPlugin {
  fn should_track_module_exports_access(parser: &JavascriptParser) -> bool {
    let is_strict = parser
      .javascript_options
      .override_strict
      .map_or(parser.build_info.strict, |strict| {
        matches!(strict, OverrideStrict::Strict)
      });

    parser.module_type.is_js_auto()
      && !parser.build_meta.esm()
      && is_strict
      && parser
        .javascript_options
        .commonjs
        .as_ref()
        .map_or(JavascriptParserCommonjsExportsOption::Enable, |commonjs| {
          commonjs.exports
        })
        != JavascriptParserCommonjsExportsOption::Disable
  }

  fn mark_module_exports_accessed(parser: &mut JavascriptParser) {
    if parser.build_info.module_exports_accessed.is_some() {
      parser.build_info.module_exports_accessed = Some(true);
    }
  }

  fn tag_factory_escape_bindings(parser: &mut JavascriptParser) {
    if parser.build_info.module_exports_accessed != Some(false) {
      return;
    }

    // `CallHooksName` dispatches variable tags before the original name. This gives the observer
    // hooks below a chance to record an access before an earlier CommonJS import or AMD hook
    // SyncBails on `require` or `define`. The observer returns `None`, so the original name is still
    // dispatched and dependency parsing remains unchanged.
    let require = Atom::from("require");
    if !parser.is_variable_defined(&require) {
      parser.tag_variable_without_data(require, COMMONJS_REQUIRE_ACCESS_TAG);
    }

    if parser.compiler_options.amd.is_some() {
      let define = Atom::from("define");
      if !parser.is_variable_defined(&define) {
        parser.tag_variable_without_data(define, COMMONJS_DEFINE_ACCESS_TAG);
      }
    }
  }

  fn is_name_or_member(for_name: &str, name: &[u8]) -> bool {
    let bytes = for_name.as_bytes();
    (bytes.len() == name.len() && bytes == name)
      || (bytes.len() > name.len() && bytes[name.len()] == b'.' && bytes.starts_with(name))
  }

  fn observe_factory_binding(&self, parser: &mut JavascriptParser, for_name: &str) {
    if parser.build_info.module_exports_accessed != Some(false) {
      return;
    }

    let accessed = for_name == COMMONJS_REQUIRE_ACCESS_TAG
      || match for_name.as_bytes().first() {
        Some(b'm') => Self::is_name_or_member(for_name, b"module"),
        Some(b'e') => Self::is_name_or_member(for_name, b"exports"),
        Some(b'a') => parser.is_top_level_this() && Self::is_name_or_member(for_name, b"arguments"),
        Some(b'r') => Self::is_name_or_member(for_name, b"require"),
        Some(b't') => parser.is_top_level_this() && Self::is_name_or_member(for_name, b"this"),
        Some(b'_') => Self::is_name_or_member(for_name, b"__webpack_module__"),
        _ => false,
      };

    if accessed {
      Self::mark_module_exports_accessed(parser);
    }
  }

  fn is_amd_define_call(call_expr: &CallExpr) -> bool {
    if call_expr
      .args
      .iter()
      .any(|argument| argument.spread.is_some())
    {
      return false;
    }

    match call_expr.args.as_slice() {
      [_] | [_, _] => true,
      [name, dependencies, _] => name.expr.is_lit() && dependencies.expr.is_array(),
      _ => false,
    }
  }

  fn observe_call(&self, parser: &mut JavascriptParser, call_expr: &CallExpr, for_name: &str) {
    self.observe_factory_binding(parser, for_name);
    if parser.build_info.module_exports_accessed != Some(false) {
      return;
    }

    let accesses_module_exports = match for_name {
      "eval" => true,
      COMMONJS_DEFINE_ACCESS_TAG => Self::is_amd_define_call(call_expr),
      "Object.defineProperty" => call_expr.args.first().is_some_and(|argument| {
        if argument.spread.is_some() {
          return false;
        }
        let evaluated = parser.evaluate_expression(&argument.expr);
        if !evaluated.is_identifier() {
          return false;
        }
        match evaluated.identifier().as_str() {
          "exports" | "module.exports" => true,
          "this" => parser.is_top_level_this(),
          _ => false,
        }
      }),
      _ => false,
    };

    if accesses_module_exports {
      Self::mark_module_exports_accessed(parser);
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CommonJsPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, _ast: &Program) -> Option<bool> {
    parser.build_info.module_exports_accessed =
      Self::should_track_module_exports_access(parser).then_some(false);
    None
  }

  fn statement(&self, parser: &mut JavascriptParser<'p>, statement: Statement<'_>) -> Option<bool> {
    // Statement walking starts after scope pre-walking, so shadowed bindings are already known.
    Self::tag_factory_escape_bindings(parser);
    if matches!(statement, Statement::Return(_)) && parser.is_top_level_scope() {
      parser.build_info.module_concatenation_bailout = Some("top-level return".into());
    }
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if for_name == expr_name::MODULE_HOT {
      Some(evaluate_to_identifier(
        expr_name::MODULE_HOT.into(),
        expr_name::MODULE.into(),
        None,
        start,
        end,
      ))
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
    self.observe_factory_binding(parser, for_name);

    if for_name == expr_name::MODULE {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span.into(),
        "'object'".into(),
      )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);

    if for_name == "module.id" {
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::MODULE_ID,
      )));
      parser.build_info.module_concatenation_bailout = Some(for_name.to_string());
      return Some(true);
    }

    if for_name == "module.loaded" {
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::MODULE_LOADED,
      )));
      parser.build_info.module_concatenation_bailout = Some(for_name.to_string());
      return Some(true);
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &ThisExpr,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_call(parser, call_expr, for_name);
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &AssignExpr,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &AssignExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &MemberExpr,
    for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &CallExpr,
    for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    self.observe_call(parser, expr, for_name);
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _member_expr: &MemberExpr,
    _callee_members: &[Atom],
    call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    self.observe_call(parser, call_expr, for_name);
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _call_expr: &CallExpr,
    _callee_members: &[Atom],
    inner_call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    self.observe_call(parser, inner_call_expr, for_name);
    None
  }
}
