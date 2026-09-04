use std::sync::Arc;

use rspack_core::{
  ConstDependency, JavascriptParserCommonjsExportsOption, OverrideStrict, RuntimeGlobals,
  RuntimeRequirementsDependency,
};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, CallExpr, Ident, MemberExpr, NewExpr, Program, Span, UnaryExpr,
};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier},
  visitors::{JavascriptParser, Statement, expr_name},
};

pub struct CommonJsPlugin;

const COMMONJS_REQUIRE_ACCESS_TAG: &str = "commonjs require access";

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

  fn tag_require_access(parser: &mut JavascriptParser) {
    if parser.build_info.module_exports_accessed != Some(false) {
      return;
    }

    // `CallHooksName` dispatches variable tags before the original name. This gives the observer
    // hooks below a chance to record an access before an earlier CommonJS or AMD hook SyncBails on
    // `require`. The observer returns `None`, so dependency parsing remains unchanged.
    let require = Atom::from("require");
    if !parser.is_variable_defined(&require) {
      parser.tag_variable_without_data(require, COMMONJS_REQUIRE_ACCESS_TAG);
    }
  }

  fn observe_factory_binding(&self, parser: &mut JavascriptParser, for_name: &str) {
    if parser.build_info.module_exports_accessed != Some(false) {
      return;
    }

    let accessed = match for_name.split('.').next() {
      Some(
        COMMONJS_REQUIRE_ACCESS_TAG | "module" | "exports" | "require" | "__webpack_module__",
      ) => true,
      Some("arguments" | "this") => parser.is_top_level_this(),
      _ => false,
    };

    if accessed {
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
    Self::tag_require_access(parser);
    if matches!(statement, Statement::Return(_)) && parser.is_top_level_scope() {
      parser.build_info.module_concatenation_bailout = Some("top-level return".into());
    }
    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    // CommonJS exports and AMD define parsing share this state. Reuse it instead of duplicating
    // their syntax handling here.
    if parser.parser_exports_state.is_some() {
      Self::mark_module_exports_accessed(parser);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    // Evaluation is also used for generated expressions such as DefinePlugin values, which may
    // be consumed without going through the normal identifier walker.
    self.observe_factory_binding(parser, for_name);

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

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    _call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
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
    _expr: &CallExpr,
    for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _member_expr: &MemberExpr,
    _callee_members: &[Atom],
    _call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _call_expr: &CallExpr,
    _callee_members: &[Atom],
    _inner_call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    self.observe_factory_binding(parser, for_name);
    None
  }
}
