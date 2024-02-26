use rspack_core::{ConstDependency, DependencyLocation, RuntimeGlobals, SpanExt};
use swc_core::common::Spanned;

use super::JavascriptParserPlugin;
use crate::visitors::{JavascriptParser, TagInfoData};

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
      parser.tag_variable(
        fn_decl.ident.sym.clone(),
        NESTED_WEBPACK_IDENTIFIER_TAG,
        Some(data),
      );
      Some(true)
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    _for_name: &str,
  ) -> Option<bool> {
    // FIXME: add condition: `if _for_name != NESTED_WEBPACK_IDENTIFIER_TAG then return None else continue`
    let name = ident.sym.as_str();
    if name != RuntimeGlobals::REQUIRE.name() {
      return None;
    }
    let mut deps = vec![];
    let Some(variable_info) = parser.get_mut_variable_info(&ident.sym) else {
      return None;
    };
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
    if !nested_require_data.update {
      deps.push(ConstDependency::new(
        nested_require_data.loc.start(),
        nested_require_data.loc.end(),
        nested_require_data.name.clone().into(),
        None,
      ));
      nested_require_data.update = true;
    }
    variable_info.update_tag_info_data(Some(NestedRequireData::serialize(&nested_require_data)));
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
}
