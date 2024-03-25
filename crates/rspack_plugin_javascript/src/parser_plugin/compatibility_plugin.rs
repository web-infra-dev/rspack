use rspack_core::{
  ConstDependency, ContextDependency, DependencyLocation, RuntimeGlobals, SpanExt,
};
use swc_core::{common::Spanned, ecma::ast::CallExpr};

use super::JavascriptParserPlugin;
use crate::{
  dependency::CommonJsRequireContextDependency,
  visitors::{expr_name, JavascriptParser, TagInfoData},
};

const NESTED_WEBPACK_IDENTIFIER_TAG: &str = "_identifier__nested_webpack_identifier__";

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct NestedRequireData {
  name: String,
  update: bool,
  loc: DependencyLocation,
}

impl TagInfoData for NestedRequireData {
  fn serialize(data: &Self) -> serde_json::Value {
    serde_json::to_value(data).expect("serialize failed for `NestedRequireData`")
  }

  fn deserialize(value: serde_json::Value) -> Self {
    serde_json::from_value(value).expect("deserialize failed for `NestedRequireData`")
  }
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
  fn pre_statement(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::Stmt,
  ) -> Option<bool> {
    let Some(fn_decl) = stmt.as_decl().and_then(|decl| decl.as_fn_decl()) else {
      return None;
    };
    let name = fn_decl.ident.sym.as_str();
    if name != RuntimeGlobals::REQUIRE.name() {
      None
    } else {
      let low = fn_decl.span().lo().0;
      let hi = fn_decl.span().hi().0;
      let data = NestedRequireData {
        name: format!("__nested_webpack_require_{low}_{hi}__"),
        update: false,
        loc: DependencyLocation::new(
          fn_decl.ident.span().real_lo(),
          fn_decl.ident.span().real_hi(),
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
    let name = ident.sym.as_str();
    if name != RuntimeGlobals::REQUIRE.name() {
      return None;
    }
    let Some(variable_info) = parser.get_mut_variable_info(name) else {
      return None;
    };

    // FIXME: should find the `tag_info` which tag equal `NESTED_WEBPACK_IDENTIFIER_TAG`;
    let Some(tag_info) = &mut variable_info.tag_info else {
      unreachable!();
    };
    if tag_info.tag != NESTED_WEBPACK_IDENTIFIER_TAG {
      return None;
    }
    let Some(data) = tag_info.data.as_mut().map(std::mem::take) else {
      unreachable!();
    };
    let mut nested_require_data = NestedRequireData::deserialize(data);
    let mut deps = Vec::with_capacity(2);
    if !nested_require_data.update {
      deps.push(ConstDependency::new(
        nested_require_data.loc.start(),
        nested_require_data.loc.end(),
        nested_require_data.name.clone().into(),
        None,
      ));
      nested_require_data.update = true;
    }
    tag_info.data = Some(NestedRequireData::serialize(&nested_require_data));

    deps.push(ConstDependency::new(
      ident.span.real_lo(),
      ident.span.real_hi(),
      nested_require_data.name.into(),
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
