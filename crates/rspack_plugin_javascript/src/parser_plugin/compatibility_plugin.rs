use rspack_core::{
  ConstDependency, ContextDependency, DependencyLocation, RuntimeGlobals, SpanExt,
};
use swc_core::{common::Spanned, ecma::ast::CallExpr};

use super::JavascriptParserPlugin;
use crate::{
  dependency::CommonJsRequireContextDependency,
  visitors::{expr_name, JavascriptParser, Statement, TagInfoData},
};

const NESTED_WEBPACK_IDENTIFIER_TAG: &str = "_identifier__nested_webpack_identifier__";

#[derive(Debug, Clone)]
struct NestedRequireData {
  name: String,
  update: bool,
  loc: DependencyLocation,
}

pub struct CompatibilityPlugin;

impl CompatibilityPlugin {
  pub fn browserify_require_handler(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
  ) -> Option<bool> {
    if expr.args.len() != 2 {
      return None;
    }
    let second = parser.evaluate_expression(&expr.args[1].expr);
    if !second.is_bool() || !matches!(second.as_bool(), Some(true)) {
      return None;
    }
    let dep = ConstDependency::new(
      expr.callee.span().real_lo(),
      expr.callee.span().real_hi(),
      "require".into(),
      None,
    );
    if let Some(last) = parser.dependencies.last()
      && let Some(last) = last.downcast_ref::<CommonJsRequireContextDependency>()
      && let options = last.options()
      && options.recursive
      && options.request == "."
    {
      parser.dependencies.pop();
      // TODO: dependency getWarnings getErrors
      parser.warning_diagnostics.pop();
    }
    parser.presentational_dependencies.push(Box::new(dep));
    Some(true)
  }
}

impl JavascriptParserPlugin for CompatibilityPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    if ast
      .as_module()
      .and_then(|m| m.shebang.as_ref())
      .or_else(|| ast.as_script().and_then(|s| s.shebang.as_ref()))
      .is_some()
    {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(0, 0, "//".into(), None)));
    }

    None
  }

  fn pattern(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == RuntimeGlobals::EXPORTS.name() {
      parser.tag_variable(
        ident.sym.to_string(),
        NESTED_WEBPACK_IDENTIFIER_TAG,
        Some(NestedRequireData {
          name: "__nested_webpack_exports__".to_string(),
          update: false,
          loc: DependencyLocation::new(
            ident.span().real_lo(),
            ident.span().real_hi(),
            Some(parser.source_map.clone()),
          ),
        }),
      );
      return Some(true);
    } else if for_name == RuntimeGlobals::REQUIRE.name() {
      let low = ident.span().lo().0;
      let hi = ident.span().hi().0;
      parser.tag_variable(
        ident.sym.to_string(),
        NESTED_WEBPACK_IDENTIFIER_TAG,
        Some(NestedRequireData {
          name: format!("__nested_webpack_require_{low}_{hi}__"),
          update: false,
          loc: DependencyLocation::new(
            ident.span().real_lo(),
            ident.span().real_hi(),
            Some(parser.source_map.clone()),
          ),
        }),
      );
      return Some(true);
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    let Some(fn_decl) = stmt.as_function_decl() else {
      return None;
    };
    let Some(ident) = fn_decl.ident else {
      return None;
    };
    let name = ident.sym.as_str();
    if name != RuntimeGlobals::REQUIRE.name() {
      None
    } else {
      let low = fn_decl.span().lo().0;
      let hi = fn_decl.span().hi().0;
      let data = NestedRequireData {
        name: format!("__nested_webpack_require_{low}_{hi}__"),
        update: false,
        loc: DependencyLocation::new(
          ident.span().real_lo(),
          ident.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
      };
      parser.tag_variable(name.to_string(), NESTED_WEBPACK_IDENTIFIER_TAG, Some(data));
      Some(true)
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != NESTED_WEBPACK_IDENTIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_mut_tag_info(&parser.current_tag_info?);

    let mut nested_require_data = NestedRequireData::downcast(tag_info.data.take()?);
    let mut deps = Vec::with_capacity(2);
    let name = nested_require_data.name.clone();
    if !nested_require_data.update {
      deps.push(ConstDependency::new(
        nested_require_data.loc.start(),
        nested_require_data.loc.end(),
        name.clone().into(),
        None,
      ));
      nested_require_data.update = true;
    }
    tag_info.data = Some(NestedRequireData::into_any(nested_require_data));

    deps.push(ConstDependency::new(
      ident.span.real_lo(),
      ident.span.real_hi(),
      name.into(),
      None,
    ));
    for dep in deps {
      parser.presentational_dependencies.push(Box::new(dep));
    }
    Some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      return self.browserify_require_handler(parser, expr);
    }
    None
  }
}
