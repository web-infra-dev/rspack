use camino::Utf8PathBuf;
use rspack_core::{ConstDependency, SpanExt};
use rspack_plugin_javascript::{
  utils::{
    self,
    eval::{self},
  },
  visitors::JavascriptParser,
  JavascriptParserPlugin,
};
use rspack_util::{atom::Atom, json_stringify};
use swc_core::{
  common::{Span, Spanned},
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

#[derive(PartialEq)]
enum ModulePathType {
  DirName,
  FileName,
}

#[derive(Debug, Default)]
pub struct RstestParserPlugin {
  pub module_path_name: bool,
  pub hoist_mock_module: bool,
  pub import_meta_path_name: bool,
  pub manual_mock_root: String,
}

impl RstestParserPlugin {
  pub fn new(
    module_path_name: bool,
    hoist_mock_module: bool,
    import_meta_path_name: bool,
    manual_mock_root: String,
  ) -> Self {
    Self {
      module_path_name,
      hoist_mock_module,
      import_meta_path_name,
      manual_mock_root,
    }
  }

  fn process_hoist_mock(&self, parser: &mut JavascriptParser, call_expr: &CallExpr) {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            parser
              .presentational_dependencies
              .push(Box::new(MockHoistDependency::new(
                call_expr.span(),
                call_expr.callee.span(),
                lit.value.to_string(),
              )));

            // Mock to alongside.
            // node:foo will be mocked to `__mocks__/foo`.
            let path_buf = Utf8PathBuf::from(
              lit
                .value
                .to_string()
                .strip_prefix("node:")
                .unwrap_or(lit.value.as_ref())
                .to_string(),
            );
            let is_relative_request = path_buf.starts_with("."); // TODO: consider alias?

            let mocked_target = if is_relative_request {
              // Mock relative request to alongside `__mocks__` directory.
              path_buf
                .parent()
                .map(|p| {
                  p.join("__mocks__")
                    .join(path_buf.file_name().unwrap_or_default())
                })
                .unwrap_or_else(|| Utf8PathBuf::from("__mocks__").join(path_buf))
            } else {
              // Mock non-relative request to `manual_mock_root` directory.
              Utf8PathBuf::from(&self.manual_mock_root).join(&path_buf)
            };

            // __webpack_require__.set_mock({a ,}{b});
            // {a, }
            if let Some(alongside_mock_request) = mocked_target.as_std_path().to_str() {
              parser
                .dependencies
                .push(Box::new(MockModuleIdDependency::new(
                  alongside_mock_request.to_string(),
                  first_arg.span().into(),
                  false,
                  true,
                  rspack_core::DependencyCategory::Esm,
                  Some(", ".to_string()),
                )));
            }

            // {b}
            let span2 = Span::new(
              first_arg.span().hi() + swc_core::common::BytePos(0),
              first_arg.span().hi() + swc_core::common::BytePos(0),
            );
            parser
              .dependencies
              .push(Box::new(MockModuleIdDependency::new(
                lit.value.to_string(),
                span2.into(),
                false,
                true,
                rspack_core::DependencyCategory::Esm,
                None,
              )));
          }
        }
      }
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
                true,
                rspack_core::DependencyCategory::Esm,
                // MockType::MockFactory,
                // lit.value.to_string(),
                None,
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
  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    _for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.hoist_mock_module {
      let expr = call_expr.callee.as_expr();
      if let Some(expr) = expr {
        let q = expr.as_member();
        if let Some(q) = q {
          if let Some(ident) = q.obj.as_ident() {
            if let Some(prop) = q.prop.as_ident() {
              if ident.sym == "rs" && prop.sym == "mock" {
                // Hoist mock module.
                self.process_hoist_mock(parser, call_expr);
                return Some(false);
              } else {
                // Not a mock module, continue.
                return None;
              }
            }
          }
        } else {
          return None;
        }
      }
    }
    None
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

    if self.module_path_name {
      match str {
        DIR_NAME => {
          parser
            .presentational_dependencies
            .push(Box::new(ModulePathNameDependency::new(NameType::DirName)));
          return Some(true);
        }
        FILE_NAME => {
          parser
            .presentational_dependencies
            .push(Box::new(ModulePathNameDependency::new(NameType::FileName)));
          return Some(true);
        }
        _ => return None,
      }
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<utils::eval::BasicEvaluatedExpression<'a>> {
    if self.import_meta_path_name {
      let mut evaluated = None;
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        evaluated = Some("string".to_string());
      }
      return evaluated
        .map(|e| eval::evaluate_to_string(e, expr.span.real_lo(), expr.span.real_hi()));
    }

    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    if self.import_meta_path_name {
      if ident == IMPORT_META_DIRNAME {
        return Some(eval::evaluate_to_string(
          self.process_import_meta(parser, ModulePathType::DirName),
          start,
          end,
        ));
      } else if ident == IMPORT_META_FILENAME {
        return Some(eval::evaluate_to_string(
          self.process_import_meta(parser, ModulePathType::FileName),
          start,
          end,
        ));
      } else {
        return None;
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().into(),
            "'string'".into(),
            None,
          )));
        return Some(true);
      } else {
        return None;
      }
    }

    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME {
        let result = self.process_import_meta(parser, ModulePathType::DirName);
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            result.into(),
            None,
          )));
        return Some(true);
      } else if for_name == IMPORT_META_FILENAME {
        let result = self.process_import_meta(parser, ModulePathType::FileName);
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            member_expr.span().into(),
            result.into(),
            None,
          )));
        return Some(true);
      } else {
        return None;
      }
    }

    None
  }
}
