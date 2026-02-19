use itertools::Itertools;
use rspack_core::{ConstDependency, property_access};
use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{
  CallExpr, Expr, GetSpan, MemberExpr, MemberProp, MetaPropKind, Span, UnaryExpr,
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  dependency::{ImportMetaResolveDependency, ImportMetaResolveHeaderDependency},
  utils::eval,
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo, RootName,
    create_traceable_error, expr_name,
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

  fn process_import_meta_resolve(&self, parser: &mut JavascriptParser, call_expr: CallExpr) {
    if call_expr.args(&parser.ast).len() != 1 {
      return;
    }

    let argument_expr = call_expr
      .args(&parser.ast)
      .get_node(&parser.ast, 0)
      .unwrap()
      .expr(&parser.ast);
    let param = parser.evaluate_expression(argument_expr);
    let import_meta_resolve_header_dependency = Box::new(ImportMetaResolveHeaderDependency::new(
      call_expr.callee(&parser.ast).span(&parser.ast).into(),
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
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: UnaryExpr,
    for_name: &str,
  ) -> Option<eval::BasicEvaluatedExpression> {
    let mut evaluated = None;
    if for_name == expr_name::IMPORT_META {
      evaluated = Some("object".to_string());
    } else if for_name == expr_name::IMPORT_META_URL {
      evaluated = Some("string".to_string());
    } else if for_name == expr_name::IMPORT_META_RESOLVE {
      evaluated = Some("function".to_string());
    } else if for_name == expr_name::IMPORT_META_VERSION {
      evaluated = Some("number".to_string())
    } else if let Some(member_expr) = expr.arg(&parser.ast).as_member()
      && let Some(meta_expr) = member_expr.obj(&parser.ast).as_meta_prop()
      && meta_expr
        .get_root_name(&parser.ast)
        .is_some_and(|name| name == expr_name::IMPORT_META)
      && (match member_expr.prop(&parser.ast) {
        MemberProp::Ident(_) => true,
        MemberProp::Computed(computed) => computed.expr(&parser.ast).is_lit(),
        _ => false,
      })
    {
      evaluated = Some("undefined".to_string())
    }
    evaluated.map(|e| {
      eval::evaluate_to_string(
        e,
        expr.span(&parser.ast).real_lo(),
        expr.span(&parser.ast).real_hi(),
      )
    })
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression> {
    if for_name == expr_name::IMPORT_META_VERSION {
      Some(eval::evaluate_to_number(5_f64, start, end))
    } else if for_name == expr_name::IMPORT_META_URL {
      Some(eval::evaluate_to_string(
        self.import_meta_url(parser),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn evaluate(
    &self,
    parser: &mut JavascriptParser,
    expr: Expr,
  ) -> Option<eval::BasicEvaluatedExpression> {
    if let Some(member) = expr.as_member()
      && let Some(meta_prop) = member.obj(&parser.ast).as_meta_prop()
      && meta_prop.kind(&parser.ast) == MetaPropKind::ImportMeta
    {
      if let Some(ident) = member.prop(&parser.ast).as_ident() {
        // Skip `dirname` and `filename` - they are handled by NodeStuffPlugin
        // and may have runtime values when node.__dirname/node.__filename is false
        let sym = parser.ast.get_utf8(ident.sym(&parser.ast));
        if sym == "dirname" || sym == "filename" {
          return None;
        }
        return Some(eval::evaluate_to_undefined(
          member.span(&parser.ast).real_lo(),
          member.span(&parser.ast).real_hi(),
        ));
      }
      if let Some(computed) = member.prop(&parser.ast).as_computed()
        && computed.expr(&parser.ast).is_lit()
      {
        // Check for computed properties like import.meta["dirname"]
        if let Some(str_lit) = computed
          .expr(&parser.ast)
          .as_lit()
          .and_then(|lit| lit.as_str())
          && (parser.ast.get_wtf8(str_lit.value(&parser.ast)) == "dirname"
            || parser.ast.get_wtf8(str_lit.value(&parser.ast)) == "filename")
        {
          return None;
        }
        return Some(eval::evaluate_to_undefined(
          member.span(&parser.ast).real_lo(),
          member.span(&parser.ast).real_hi(),
        ));
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      expr_name::IMPORT_META => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span(&parser.ast).into(),
          "'object'".into(),
        )));
        Some(true)
      }
      expr_name::IMPORT_META_URL => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span(&parser.ast).into(),
          "'string'".into(),
        )));
        Some(true)
      }
      expr_name::IMPORT_META_RESOLVE => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span(&parser.ast).into(),
          "'function'".into(),
        )));
        Some(true)
      }
      expr_name::IMPORT_META_VERSION => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span(&parser.ast).into(),
          "'number'".into(),
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    _parser: &mut JavascriptParser,
    expr: Expr,
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
            content.push(format!(r#"url: "{}""#, self.import_meta_url(parser)))
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
    member_expr: MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::IMPORT_META_URL {
      // import.meta.url
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span(&parser.ast).into(),
        format!("'{}'", self.import_meta_url(parser)).into(),
      )));
      Some(true)
    } else if for_name == expr_name::IMPORT_META_VERSION {
      // import.meta.webpack
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span(&parser.ast).into(),
        self.import_meta_version().into(),
      )));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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
    expr: MemberExpr,
  ) -> Option<bool> {
    match root_info {
      ExportedVariableInfo::Name(root) => {
        if root == expr_name::IMPORT_META {
          let members = parser
            .get_member_expression_info(Expr::Member(expr), AllowedMemberTypes::Expression)
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
              ConstDependency::new(expr.span(&parser.ast).into(), "undefined".into())
            } else {
              ConstDependency::new(
                expr.span(&parser.ast).into(),
                self
                  .import_meta_unknown_property(
                    &members.members.iter().map(|x| x.to_string()).collect_vec(),
                  )
                  .into(),
              )
            }
          } else {
            ConstDependency::new(expr.span(&parser.ast).into(), "undefined".into())
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
      )));
      Some(true)
    } else {
      None
    }
  }
}
