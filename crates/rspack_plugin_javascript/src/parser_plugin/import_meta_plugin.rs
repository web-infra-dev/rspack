// import_meta_plugin.rs
use rspack_core::{ConstDependency, SpanExt};
use rspack_error::miette::Severity;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::Ident;
use url::Url;

use super::JavascriptParserPlugin;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::expr_name;
use crate::visitors::ExportedVariableInfo;
use crate::visitors::{create_traceable_error, JavascriptParser};

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/ImportMetaPlugin.js
pub struct ImportMetaPlugin;

const WEBPACK_VERSION: i32 = 5;
const RSPACK_VERSION: i32 = 0;

impl JavascriptParserPlugin for ImportMetaPlugin {
  fn r#typeof(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    println!("typeof: {:?}", for_name);
    let content;
    if for_name == expr_name::IMPORT_META {
      content = "'object'";
    } else if for_name == expr_name::IMPORT_META_URL {
      content = "'string'";
    } else if for_name == expr_name::IMPORT_META_WEBPACK
      || for_name == expr_name::IMPORT_META_RSPACK
    {
      content = "'number'";
    } else if for_name.starts_with(expr_name::IMPORT_META_ENV) {
      content = "'boolean'";
    } else {
      return None;
    }

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        unary_expr.span().real_lo(),
        unary_expr.span().real_hi(),
        content.into(),
        None,
      )));

    Some(true)
  }

  fn meta_property(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    println!("meta_property: {:?}", root_name);
    if root_name == expr_name::IMPORT_META {
      // import.meta
      // warn when access import.meta directly
      parser.warning_diagnostics.push(Box::new(create_traceable_error(
      "Critical dependency".into(),
      "Accessing import.meta directly is unsupported (only property access or destructuring is supported)".into(),
      parser.source_file,
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
    println!("member: {:?}", for_name);
    let content;
    if for_name.starts_with(expr_name::IMPORT_META_ENV) {
      // import.meta.env
      content = "false".to_string();
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      // import.meta.webpack
      content = format!("{WEBPACK_VERSION}");
    } else if for_name.starts_with(expr_name::IMPORT_META_URL) {
      // import.meta.url
      let url = Url::from_file_path(&parser.resource_data.resource).expect("should be a path");
      content = for_name.replace(expr_name::IMPORT_META_URL, &format!("'{url}'"));
    } else if for_name == expr_name::IMPORT_META_RSPACK {
      // import.meta.rspack
      content = format!("{RSPACK_VERSION}");
    } else if for_name.starts_with(expr_name::IMPORT_META) {
      // import.meta.xx or import.meta.xx.xx
      let mut parts: Vec<&str> = for_name.split('.').collect();
      if parts.len() == 2 {
        return None;
      }

      let void_str = if parts.len() > 3 {
        "(void 0)"
      } else {
        "void 0"
      };

      parts.splice(0..3, vec![void_str].into_iter());
      content = parts.join(".");
    } else {
      return None;
    }

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        member_expr.span().real_lo(),
        member_expr.span().real_hi(),
        content.into(),
        None,
      )));

    Some(true)
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    println!("evaluate_typeof: {:?}", _ident);
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut crate::visitors::JavascriptParser,
    _ident: &str,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    println!("evaluate_identifier: {:?}", _ident);
    None
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &swc_core::ecma::ast::MemberExpr,
  ) -> Option<bool> {
    dbg!("unhandled_expression_member_chain {:?}", &expr);
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
