use rspack_core::{ConstDependency, SpanExt};
use rspack_error::miette::Severity;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::MemberProp;
use url::Url;

use super::JavascriptParserPlugin;
use crate::utils::eval;
use crate::visitors::expr_name;
use crate::visitors::ExportedVariableInfo;
use crate::visitors::JavascriptParser;
use crate::visitors::{create_traceable_error, RootName};

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/ImportMetaPlugin.js
// TODO:
// - scan `import.meta.url.indexOf("index.js")`
// - evaluate expression. eg `import.meta.env && import.meta.env.xx` should be `false`
pub struct ImportMetaPlugin;

impl JavascriptParserPlugin for ImportMetaPlugin {
  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    let mut evaluated = None;
    if for_name == expr_name::IMPORT_META {
      evaluated = Some("object".to_string());
    } else if for_name == expr_name::IMPORT_META_URL {
      evaluated = Some("string".to_string());
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      evaluated = Some("number".to_string())
    } else if let Some(member_expr) = expr.arg.as_member()
      && let Some(meta_expr) = member_expr.obj.as_meta_prop()
      && meta_expr
        .get_root_name()
        .is_some_and(|name| name == expr_name::IMPORT_META)
      && (match &member_expr.prop {
        MemberProp::Ident(_) => true,
        MemberProp::Computed(computed) => computed.expr.is_lit(),
        _ => false,
      })
    {
      evaluated = Some("undefined".to_string())
    }
    evaluated.map(|e| eval::evaluate_to_string(e, expr.span.real_lo(), expr.span.real_hi()))
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression> {
    if ident == expr_name::IMPORT_META_WEBPACK {
      Some(eval::evaluate_to_number(5_f64, start, end))
    } else {
      None
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
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
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().real_lo(),
          unary_expr.span().real_hi(),
          "'number'".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    if root_name == expr_name::IMPORT_META {
      // import.meta
      // warn when access import.meta directly
      parser.warning_diagnostics.push(Box::new(create_traceable_error(
      "Critical dependency".into(),
      "Accessing import.meta directly is unsupported (only property access or destructuring is supported)".into(),
      parser.source_file,
      span.into()
    ).with_severity(Severity::Warning)));

      let content = if parser.is_asi_position(span.lo()) {
        ";({})"
      } else {
        "({})"
      };
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          span.real_lo(),
          span.real_hi(),
          content.into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
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
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      // import.meta.webpack
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().real_lo(),
          member_expr.span().real_hi(),
          "5".to_string().into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser,
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
