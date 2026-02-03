use itertools::Itertools;
use rspack_core::{ConstDependency, property_access};
use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use swc_core::{
  common::{Span, Spanned},
  ecma::ast::{Expr, MemberProp, MetaPropKind},
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  dependency::{ImportMetaResolveDependency, ImportMetaResolveHeaderDependency},
  utils::eval,
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, ExprRef, JavascriptParser, MemberExpressionInfo,
    RootName, create_traceable_error, expr_name,
  },
};

pub struct ImportMetaPlugin;

impl ImportMetaPlugin {
  fn import_meta_url(&self, parser: &JavascriptParser) -> String {
    Url::from_file_path(parser.resource_data.resource())
      .expect("should be a path")
      .to_string()
  }

  fn import_meta_version(&self) -> String {
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
      Some(parser.source()),
    ));

    if param.is_conditional() {
      for option in param.options() {
        self.process_import_meta_resolve_item(parser, option);
      }
    } else {
      self.process_import_meta_resolve_item(parser, &param);
    }
    parser.add_dependency(import_meta_resolve_header_dependency);
  }

  fn process_import_meta_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &eval::BasicEvaluatedExpression,
  ) {
    if param.is_string() {
      let (start, end) = param.range();
      parser.add_dependency(Box::new(ImportMetaResolveDependency::new(
        param.string().clone(),
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
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    let mut evaluated = None;
    if for_name == expr_name::IMPORT_META {
      evaluated = Some("object".to_string());
    } else if for_name == expr_name::IMPORT_META_URL {
      evaluated = Some("string".to_string());
    } else if for_name == expr_name::IMPORT_META_RESOLVE {
      evaluated = Some("function".to_string());
    } else if for_name == expr_name::IMPORT_META_VERSION {
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
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::IMPORT_META_VERSION {
      Some(eval::evaluate_to_number(5_f64, start, end))
    } else if for_name == expr_name::IMPORT_META_URL {
      if parser.is_esm {
        if parser
          .compiler_options
          .output
          .environment
          .supports_document()
        {
          None
        } else {
          Some(eval::evaluate_to_string(
            self.import_meta_url(parser),
            start,
            end,
          ))
        }
      } else {
        None
      }
    } else {
      None
    }
  }

  fn evaluate<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    if let Some(member) = expr.as_member()
      && let Some(meta_prop) = member.obj.as_meta_prop()
      && meta_prop.kind == MetaPropKind::ImportMeta
    {
      if let Some(ident) = member.prop.as_ident() {
        // Skip `dirname` and `filename` - they are handled by NodeStuffPlugin
        // and may have runtime values when node.__dirname/node.__filename is false
        if ident.sym == "dirname" || ident.sym == "filename" {
          return None;
        }
        return Some(eval::evaluate_to_undefined(
          member.span().real_lo(),
          member.span().real_hi(),
        ));
      }
      if let Some(computed) = member.prop.as_computed()
        && computed.expr.is_lit()
      {
        // Check for computed properties like import.meta["dirname"]
        if let Some(str_lit) = computed.expr.as_lit().and_then(|lit| lit.as_str())
          && (str_lit.value == "dirname" || str_lit.value == "filename")
        {
          return None;
        }
        return Some(eval::evaluate_to_undefined(
          member.span().real_lo(),
          member.span().real_hi(),
        ));
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      expr_name::IMPORT_META => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'object'".into(),
          None,
        )));
        Some(true)
      }
      expr_name::IMPORT_META_URL => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'string'".into(),
          None,
        )));
        Some(true)
      }
      expr_name::IMPORT_META_RESOLVE => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'function'".into(),
          None,
        )));
        Some(true)
      }
      expr_name::IMPORT_META_VERSION => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'number'".into(),
          None,
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    _parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    if expr.is_meta_prop() {
      return Some(true);
    }
    None
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    if root_name == expr_name::IMPORT_META {
      let destructuring_assignment_properties = parser
        .destructuring_assignment_properties
        .get(&span)
        .cloned();

      if let Some(referenced_properties_in_destructuring) = destructuring_assignment_properties {
        let mut content = vec![];
        for prop in referenced_properties_in_destructuring.iter() {
          let res = parser
            .plugin_drive
            .clone()
            .import_meta_property_in_destructuring(parser, prop);

          if let Some(property) = res {
            content.push(property);
            continue;
          }
          if prop.id == "url" {
            if parser.is_esm
              && parser
                .compiler_options
                .output
                .environment
                .supports_document()
            {
              // Preserve import.meta.url for web targets in ES modules using a getter
              content.push(r#"get url() { return import.meta.url; }"#.to_string())
            } else {
              content.push(format!(r#"url: "{}""#, self.import_meta_url(parser)))
            }
          } else if prop.id == "webpack" {
            content.push(format!(r#"webpack: {}"#, self.import_meta_version()));
          } else {
            content.push(format!(
              r#"[{}]: {}"#,
              serde_json::to_string(&prop.id).expect("json stringify failed"),
              self.import_meta_unknown_property(&vec![prop.id.to_string()])
            ));
          }
        }
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          span.into(),
          format!("({{{}}})", content.join(",")).into(),
          None,
        )));
        Some(true)
      } else {
        // import.meta
        // warn when access import.meta directly
        let mut error: Error = create_traceable_error(
          "Critical dependency".into(),
          "Accessing import.meta directly is unsupported (only property access or destructuring is supported)".into(),
          parser.source.to_string(),
          span.into()
        );
        error.severity = Severity::Warning;
        parser.add_warning(error.into());

        let content = if parser.is_asi_position(span.lo()) {
          ";({})"
        } else {
          "({})"
        };
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
      if !parser.is_esm {
        // import.meta.url is only available in ES modules
        return None;
      }
      if parser
        .compiler_options
        .output
        .environment
        .supports_document()
      {
        Some(true)
      } else {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          format!("'{}'", self.import_meta_url(parser)).into(),
          None,
        )));
        Some(true)
      }
    } else if for_name == expr_name::IMPORT_META_VERSION {
      // import.meta.webpack
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        self.import_meta_version().into(),
        None,
      )));
      Some(true)
    } else {
      None
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
            .get_member_expression_info(ExprRef::Member(expr), AllowedMemberTypes::Expression)
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

          parser.add_presentational_dependency(Box::new(dep));
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
