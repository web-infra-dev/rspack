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
  dependency::{ImportMetaResolveDependency, ImportMetaResolveHeaderDependency},
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
      .expect("should be a url")
      .to_string()
  }

  // This is the same as the url.fileURLToPath() of the import.meta.url
  fn import_meta_filename(&self, parser: &JavascriptParser) -> String {
    Url::from_file_path(&parser.resource_data.resource)
      .expect("should be a url")
      .to_file_path()
      .expect("should be a path")
      .to_string_lossy()
      .into_owned()
  }

  // This is the same as the path.dirname() of the import.meta.filename
  fn import_meta_dirname(&self, parser: &JavascriptParser) -> String {
    Url::from_file_path(&parser.resource_data.resource)
      .expect("should be a url")
      .to_file_path()
      .expect("should be a path")
      .parent()
      .expect("should have a parent")
      .to_string_lossy()
      .into_owned()
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

  fn process_import_meta_resolve(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &swc_core::ecma::ast::CallExpr,
  ) {
    if call_expr.args.len() != 1 {
      return;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    let import_meta_resolve_header_dependency = Box::new(ImportMetaResolveHeaderDependency::new(
      call_expr.callee.span().into(),
      Some(parser.source_map.clone()),
    ));

    if param.is_conditional() {
      for option in param.options() {
        self.process_import_meta_resolve_item(parser, option);
      }
      parser
        .dependencies
        .push(import_meta_resolve_header_dependency);
    } else {
      self.process_import_meta_resolve_item(parser, &param);
      parser
        .dependencies
        .push(import_meta_resolve_header_dependency);
    }
  }

  fn process_import_meta_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &eval::BasicEvaluatedExpression,
  ) {
    if param.is_string() {
      let (start, end) = param.range();
      parser
        .dependencies
        .push(Box::new(ImportMetaResolveDependency::new(
          param.string().to_string(),
          (start, end - 1).into(),
          parser.in_try,
        )));
    }
  }
}

impl JavascriptParserPlugin for ImportMetaPlugin {
  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'a>> {
    let evaluated = match for_name {
      expr_name::IMPORT_META => Some("object".to_string()),
      expr_name::IMPORT_META_URL
      | expr_name::IMPORT_META_FILENAME
      | expr_name::IMPORT_META_DIRNAME => Some("string".to_string()),
      expr_name::IMPORT_META_RESOLVE => Some("function".to_string()),
      expr_name::IMPORT_META_WEBPACK => Some("number".to_string()),
      _ => expr
        .arg
        .as_member()
        .and_then(|member_expr| {
          member_expr
            .obj
            .as_meta_prop()
            .filter(|meta_expr| {
              meta_expr
                .get_root_name()
                .is_some_and(|name| name == expr_name::IMPORT_META)
            })
            .map(|_| member_expr)
        })
        .filter(|member_expr| match &member_expr.prop {
          MemberProp::Ident(_) => true,
          MemberProp::Computed(computed) => computed.expr.is_lit(),
          _ => false,
        })
        .map(|_| "undefined".to_string()),
    };

    evaluated.map(|e| eval::evaluate_to_string(e, expr.span.real_lo(), expr.span.real_hi()))
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    match ident {
      expr_name::IMPORT_META_WEBPACK => Some(eval::evaluate_to_number(5_f64, start, end)),
      expr_name::IMPORT_META_URL => Some(eval::evaluate_to_string(
        self.import_meta_url(parser),
        start,
        end,
      )),
      expr_name::IMPORT_META_FILENAME => Some(eval::evaluate_to_string(
        self.import_meta_filename(parser),
        start,
        end,
      )),
      expr_name::IMPORT_META_DIRNAME => Some(eval::evaluate_to_string(
        self.import_meta_dirname(parser),
        start,
        end,
      )),
      _ => None,
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      expr_name::IMPORT_META => {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().into(),
            "'object'".into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_URL
      | expr_name::IMPORT_META_FILENAME
      | expr_name::IMPORT_META_DIRNAME => {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().into(),
            "'string'".into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_RESOLVE => {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().into(),
            "'function'".into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_WEBPACK => {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().into(),
            "'number'".into(),
            None,
          )));
        Some(true)
      }
      _ => None,
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
          match prop.id.as_str() {
            "url" => {
              content.push(format!(r#"url: "{}""#, self.import_meta_url(parser)));
            }
            "filename" => {
              content.push(format!(
                r#"filename: "{}""#,
                self.import_meta_filename(parser)
              ));
            }
            "dirname" => {
              content.push(format!(
                r#"dirname: "{}""#,
                self.import_meta_dirname(parser)
              ));
            }
            "webpack" => {
              content.push(format!(
                r#"webpack: {}"#,
                self.import_meta_webpack_version()
              ));
            }
            _ => {
              content.push(format!(
                r#"[{}]: {}"#,
                serde_json::to_string(&prop.id).expect("json stringify failed"),
                self.import_meta_unknown_property(&vec![prop.id.to_string()])
              ));
            }
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
    match for_name {
      expr_name::IMPORT_META_URL => {
        // import.meta.url
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            format!("'{}'", self.import_meta_url(parser)).into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_FILENAME => {
        // import.meta.filename
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            format!("'{}'", self.import_meta_filename(parser)).into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_DIRNAME => {
        // import.meta.dirname
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            format!("'{}'", self.import_meta_dirname(parser)).into(),
            None,
          )));
        Some(true)
      }
      expr_name::IMPORT_META_WEBPACK => {
        // import.meta.webpack
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            self.import_meta_webpack_version().into(),
            None,
          )));
        Some(true)
      }
      _ => None,
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_RESOLVE {
      self.process_import_meta_resolve(parser, call_expr);
      return Some(true);
    }
    None
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
