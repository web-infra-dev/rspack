use itertools::Itertools;
use rspack_core::{property_access, ConstDependency, SpanExt};
use rspack_error::miette::Severity;
use swc_core::{
  common::{Span, Spanned},
  ecma::ast::MemberProp,
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  utils::eval,
  visitors::{
    create_traceable_error, expr_name, AllowedMemberTypes, ExportedVariableInfo, JavascriptParser,
    MemberExpressionInfo, RootName,
  },
};

pub struct ImportMetaPlugin;

impl ImportMetaPlugin {
  fn import_meta_url(&self, parser: &JavascriptParser) -> String {
    Url::from_file_path(&parser.resource_data.resource)
      .expect("should be a path")
      .to_string()
  }

  fn import_meta_webpack_version(&self) -> String {
    "5".to_string()
  }

  fn import_meta_unknown_property(&self, members: &Vec<String>) -> String {
    format!(
      r#"/* unsupported import.meta.{} */ undefined{}"#,
      members.join("."),
      property_access(members, 1)
    )
  }
}

impl JavascriptParserPlugin for ImportMetaPlugin {
  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'a>> {
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
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    if ident == expr_name::IMPORT_META_WEBPACK {
      Some(eval::evaluate_to_number(5_f64, start, end))
    } else if ident == expr_name::IMPORT_META_URL {
      Some(eval::evaluate_to_string(
        self.import_meta_url(parser),
        start,
        end,
      ))
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
          unary_expr.span().into(),
          "'object'".into(),
          None,
        )));
      Some(true)
    } else if for_name == expr_name::IMPORT_META_URL {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'string'".into(),
          None,
        )));
      Some(true)
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().into(),
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
      if let Some(referenced_properties_in_destructuring) =
        parser.destructuring_assignment_properties_for(&span)
      {
        let mut content = vec![];
        for prop in referenced_properties_in_destructuring {
          if prop.id == "url" {
            content.push(format!(r#"url: "{}""#, self.import_meta_url(parser)))
          } else if prop.id == "webpack" {
            content.push(format!(
              r#"webpack: {}"#,
              self.import_meta_webpack_version()
            ));
          } else {
            content.push(format!(
              r#"[{}]: {}"#,
              serde_json::to_string(&prop.id).expect("json stringify failed"),
              self.import_meta_unknown_property(&vec![prop.id.to_string()])
            ));
          }
        }
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            span.into(),
            format!("({{{}}})", content.join(",")).into(),
            None,
          )));
        Some(true)
      } else {
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
            span.into(),
            content.into(),
            None,
          )));
        Some(true)
      }
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
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().into(),
          format!("'{}'", self.import_meta_url(parser)).into(),
          None,
        )));
      Some(true)
    } else if for_name == expr_name::IMPORT_META_WEBPACK {
      // import.meta.webpack
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().into(),
          self.import_meta_webpack_version().into(),
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
          let members = parser
            .get_member_expression_info(expr, AllowedMemberTypes::Expression)
            .and_then(|info| match info {
              MemberExpressionInfo::Expression(res) => Some(res),
              _ => None,
            });

          let dep = if let Some(members) = members {
            if members.members.get(1).is_some()
              && members
                .members_optionals
                .get(1)
                .is_some_and(|optional| *optional)
            {
              ConstDependency::new(expr.span().into(), "undefined".into(), None)
            } else {
              ConstDependency::new(
                expr.span().into(),
                self
                  .import_meta_unknown_property(
                    &members.members.iter().map(|x| x.to_string()).collect_vec(),
                  )
                  .into(),
                None,
              )
            }
          } else {
            ConstDependency::new(expr.span().into(), "undefined".into(), None)
          };

          parser.presentational_dependencies.push(Box::new(dep));
          return Some(true);
        }
      }
      ExportedVariableInfo::VariableInfo(_) => (),
    }
    None
  }
}

// use when parser.import_meta is false
pub struct ImportMetaDisabledPlugin;

impl JavascriptParserPlugin for ImportMetaDisabledPlugin {
  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    let import_meta_name = parser.compiler_options.output.import_meta_name.clone();
    if import_meta_name == expr_name::IMPORT_META {
      None
    } else if root_name == expr_name::IMPORT_META {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          span.into(),
          import_meta_name.into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }
}
