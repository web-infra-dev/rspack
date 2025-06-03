use rspack_core::{ConstDependency, SpanExt};
use rspack_plugin_javascript::{
  utils, utils::eval, visitors::JavascriptParser, JavascriptParserPlugin,
};
use swc_core::common::Spanned;

use crate::module_path_name_dependency::{ModulePathNameDependency, NameType};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
const IMPORT_META_FILENAME: &str = "import.meta.filename";

#[derive(PartialEq)]
enum ModulePathType {
  DirName,
  FileName,
}

#[derive(Debug, Default)]
pub struct RstestParserPlugin;

impl RstestParserPlugin {
  fn import_meta(&self, parser: &mut JavascriptParser, r#type: ModulePathType) -> String {
    if r#type == ModulePathType::FileName {
      if let Some(resource_path) = &parser.resource_data.resource_path {
        format!("'{}'", resource_path.clone().into_string())
      } else {
        "''".to_string()
      }
    } else {
      let resource_path = parser
        .resource_data
        .resource_path
        .as_deref()
        .and_then(|p| p.parent())
        .map(|p| p.to_string())
        .unwrap_or_default();
      format!("'{resource_path}'")
    }
  }
}

impl JavascriptParserPlugin for RstestParserPlugin {
  fn identifier(
    &self,
    parser: &mut rspack_plugin_javascript::visitors::JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    _for_name: &str,
  ) -> Option<bool> {
    let str = ident.sym.as_str();
    if !parser.is_unresolved_ident(str) {
      return None;
    }

    match str {
      DIR_NAME => {
        parser
          .presentational_dependencies
          .push(Box::new(ModulePathNameDependency::new(NameType::DirName)));
        Some(true)
      }
      FILE_NAME => {
        parser
          .presentational_dependencies
          .push(Box::new(ModulePathNameDependency::new(NameType::FileName)));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<utils::eval::BasicEvaluatedExpression<'a>> {
    let mut evaluated = None;
    if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
      evaluated = Some("string".to_string());
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
    if ident == IMPORT_META_DIRNAME {
      Some(eval::evaluate_to_string(
        self.import_meta(parser, ModulePathType::DirName),
        start,
        end,
      ))
    } else if ident == IMPORT_META_FILENAME {
      Some(eval::evaluate_to_string(
        self.import_meta(parser, ModulePathType::FileName),
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
    if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'string'".into(),
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
    if for_name == IMPORT_META_DIRNAME {
      let result = self.import_meta(parser, ModulePathType::DirName);
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
          None,
        )));
      Some(true)
    } else if for_name == IMPORT_META_FILENAME {
      let result = self.import_meta(parser, ModulePathType::FileName);
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }
}
