use rspack_core::{ConstDependency, SpanExt};
use rspack_plugin_javascript::{
  utils::{self, eval},
  visitors::JavascriptParser,
  JavascriptParserPlugin,
};
use rspack_util::json_stringify;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Ident, MemberExpr, UnaryExpr},
};

use crate::{
  mock_hoist_dependency::MockHoistDependency,
  mock_module_id_dependency::MockModuleIdDependency,
  module_path_name_dependency::{ModulePathNameDependency, NameType},
};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
const IMPORT_META_FILENAME: &str = "import.meta.filename";
const RS_MOCK: &str = "rs.mock";

#[derive(PartialEq)]
enum ModulePathType {
  DirName,
  FileName,
}

#[derive(Debug, Default)]
pub struct RstestParserPlugin;

impl RstestParserPlugin {
  fn process_hoist_mock(&self, parser: &mut JavascriptParser, call_expr: &CallExpr) {
    match call_expr.args.len() {
      // TODO: mock a module to __mocks__
      1 => {}
      // mock a module
      2 => {
        let first_arg = &call_expr.args[0];
        let second_arg = &call_expr.args[1];

        if first_arg.spread.is_some() || second_arg.spread.is_some() {
          return;
        }

        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            parser
              .presentational_dependencies
              .push(Box::new(MockHoistDependency::new(
                call_expr.span(),
                call_expr.callee.span(),
                lit.value.to_string(),
              )));

            parser
              .dependencies
              .push(Box::new(MockModuleIdDependency::new(
                lit.value.to_string(),
                first_arg.span().into(),
                false,
                parser.in_try,
                rspack_core::DependencyCategory::Esm,
              )));
          } else {
            panic!("`rs.mock` function expects a string literal as the first argument");
          }
        }
      }
      _ => {
        panic!("`rs.mock` function expects 1 or 2 arguments, got more than 2");
      }
    }
  }

  fn process_import_meta(&self, parser: &mut JavascriptParser, r#type: ModulePathType) -> String {
    if r#type == ModulePathType::FileName {
      if let Some(resource_path) = &parser.resource_data.resource_path {
        json_stringify(&resource_path.clone().into_string())
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
      json_stringify(&resource_path)
    }
  }
}

impl JavascriptParserPlugin for RstestParserPlugin {
  fn call(
    &self,
    parser: &mut rspack_plugin_javascript::visitors::JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == RS_MOCK {
      self.process_hoist_mock(parser, call_expr);
      Some(false)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut rspack_plugin_javascript::visitors::JavascriptParser,
    ident: &Ident,
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
    expr: &'a UnaryExpr,
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
        self.process_import_meta(parser, ModulePathType::DirName),
        start,
        end,
      ))
    } else if ident == IMPORT_META_FILENAME {
      Some(eval::evaluate_to_string(
        self.process_import_meta(parser, ModulePathType::FileName),
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
    unary_expr: &UnaryExpr,
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
    member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == IMPORT_META_DIRNAME {
      let result = self.process_import_meta(parser, ModulePathType::DirName);
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
          None,
        )));
      Some(true)
    } else if for_name == IMPORT_META_FILENAME {
      let result = self.process_import_meta(parser, ModulePathType::FileName);
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
