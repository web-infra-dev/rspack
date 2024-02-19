use rspack_core::{ConstDependency, SpanExt};
use rspack_error::miette::Severity;
use swc_core::common::{Span, Spanned};
use url::Url;

use super::JavascriptParserPlugin;
use crate::visitors::create_traceable_error;
use crate::visitors::expr_name;
use crate::visitors::ExportedVariableInfo;

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/ImportMetaPlugin.js
// TODO:
// - scan `import.meta.webpack`
// - scan `import.meta.url.indexOf("index.js")`
// - evaluate expression. eg `import.meta.env && import.meta.env.xx` should be `false`
// - add warning for `import.meta`
pub struct ImportMetaPlugin;

impl JavascriptParserPlugin for ImportMetaPlugin {
  fn r#typeof(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().real_lo(),
          unary_expr.span().real_hi(),
          "'object'".into(),
          None,
        )));
      Some(true)
    } else if for_name == expr_name::IMPORT_META_URL {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().real_lo(),
          unary_expr.span().real_hi(),
          "'string'".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn meta_property(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    if root_name == expr_name::IMPORT_META {
      // import.meta
      // warn when access import.meta directly
      parser.warning_diagnostics.push(Box::new(create_traceable_error(
      "Critical dependency".into(),
      "Accessing import.meta directly is unsupported (only property access or destructuring is supported)".into(),
      &parser.source_file,
      span.into()
    ).with_severity(Severity::Warning)));
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          span.real_lo(),
          span.real_hi(),
          "({})".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_URL {
      // import.meta.url
      let url = Url::from_file_path(&parser.resource_data.resource).expect("should be a path");
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().real_lo(),
          member_expr.span().real_hi(),
          format!("'{url}'").into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &swc_core::ecma::ast::MemberExpr,
  ) -> Option<bool> {
    match root_info {
      ExportedVariableInfo::Name(root) => {
        if root == expr_name::IMPORT_META {
          parser
            .presentational_dependencies
            .push(Box::new(ConstDependency::new(
              expr.span().real_lo(),
              expr.span().real_hi(),
              "undefined".into(),
              None,
            )));
          return Some(true);
        }
      }
      ExportedVariableInfo::VariableInfo(_) => (),
    }
    None
  }
}
