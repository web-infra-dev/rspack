use camino::Utf8PathBuf;
use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyRange, ImportAttributes, SharedSourceMap,
  SpanExt,
};
use rspack_plugin_javascript::{
  dependency::{CommonJsRequireDependency, ImportDependency, RequireHeaderDependency},
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
  mock_dependency::MockDependency,
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

  fn process_require_actual(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            let mut range_expr: DependencyRange = first_arg.span().into();
            range_expr.end += 1; // TODO: wtf?
            let dep = CommonJsRequireDependency::new(
              lit.value.to_string(),
              range_expr,
              Some(call_expr.span.into()),
              parser.in_try,
              Some(parser.source_map.clone()),
            );
            parser.dependencies.push(Box::new(dep));

            let range: DependencyRange = call_expr.callee.span().into();
            parser
              .presentational_dependencies
              .push(Box::new(RequireHeaderDependency::new(
                range,
                Some(parser.source_map.clone()),
              )));

            return Some(true);
          }
        }
      }
      _ => {
        panic!("`rs.importActual` function expects 1 argument");
      }
    }

    None
  }

  fn process_import_actual(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            let mut attrs = ImportAttributes::default();
            attrs.insert("rstest".to_string(), "importActual".to_string());
            let dep = Box::new(ImportDependency::new(
              Atom::from(lit.value.as_ref()),
              call_expr.span.into(),
              None,
              Some(attrs),
              parser.in_try,
            ));

            let source_map: SharedSourceMap = parser.source_map.clone();
            let block = AsyncDependenciesBlock::new(
              *parser.module_identifier,
              Into::<DependencyRange>::into(call_expr.span).to_loc(Some(&source_map)),
              None,
              vec![dep],
              Some(lit.value.to_string()),
            );

            parser.blocks.push(Box::new(block));
            return Some(true);
          }
        }
      }
      _ => {
        panic!("`rs.importActual` function expects 1 argument");
      }
    }

    None
  }

  fn process_mock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    hoist: bool,
    is_esm: bool,
  ) {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            parser
              .presentational_dependencies
              .push(Box::new(MockDependency::new(
                call_expr.span(),
                call_expr.callee.span(),
                lit.value.to_string(),
                hoist,
              )));

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
                  lit.value.to_string(),
                  first_arg.span().into(),
                  false,
                  true,
                  if is_esm {
                    rspack_core::DependencyCategory::Esm
                  } else {
                    rspack_core::DependencyCategory::CommonJS
                  },
                  Some(", ".to_string()),
                )));

              // {b}
              let second_arg = Span::new(
                first_arg.span().hi() + swc_core::common::BytePos(0),
                first_arg.span().hi() + swc_core::common::BytePos(0),
              );
              parser
                .dependencies
                .push(Box::new(MockModuleIdDependency::new(
                  alongside_mock_request.to_string(),
                  second_arg.into(),
                  false,
                  true,
                  if is_esm {
                    rspack_core::DependencyCategory::Esm
                  } else {
                    rspack_core::DependencyCategory::CommonJS
                  },
                  None,
                )));
            }
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
              .push(Box::new(MockDependency::new(
                call_expr.span(),
                call_expr.callee.span(),
                lit.value.to_string(),
                hoist,
              )));

            parser
              .dependencies
              .push(Box::new(MockModuleIdDependency::new(
                lit.value.to_string(),
                first_arg.span().into(),
                false,
                true,
                if is_esm {
                  rspack_core::DependencyCategory::Esm
                } else {
                  rspack_core::DependencyCategory::CommonJS
                },
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

  fn do_unmock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    is_esm: bool,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            parser
              .presentational_dependencies
              .push(Box::new(ConstDependency::new(
                call_expr.callee.span().into(),
                "__webpack_require__.do_unmock".into(),
                None,
              )));

            parser
              .dependencies
              .push(Box::new(MockModuleIdDependency::new(
                lit.value.to_string(),
                first_arg.span().into(),
                false,
                true,
                if is_esm {
                  rspack_core::DependencyCategory::Esm
                } else {
                  rspack_core::DependencyCategory::CommonJS
                },
                Some("".to_string()),
              )));
            Some(true)
          } else {
            None
          }
        } else {
          None
        }
      }
      _ => {
        panic!("`rs.doUnmock` function expects 1 argument, got more than 1");
      }
    }
  }

  fn reset_modules(&self, parser: &mut JavascriptParser, call_expr: &CallExpr) -> Option<bool> {
    match call_expr.args.len() {
      0 => {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            call_expr.callee.span().into(),
            "__webpack_require__.reset_modules".into(),
            None,
          )));
        Some(true)
      }
      _ => {
        panic!("`rs.resetModules` function expects 0 arguments, got more than 0");
      }
    }
  }

  fn load_mock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    is_esm: bool,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            // ===== copied =====
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

            if let Some(alongside_mock_request) = mocked_target.as_std_path().to_str() {
              // __webpack_require__.set_mock({a ,}{b});
              // {a, }
              // ===== copied =====

              if is_esm {
                let mut attrs = ImportAttributes::default();
                attrs.insert("rstest".to_string(), "importMock".to_string());
                let dep = Box::new(ImportDependency::new(
                  Atom::from(alongside_mock_request),
                  call_expr.span.into(),
                  None,
                  Some(attrs),
                  parser.in_try,
                ));

                let source_map: SharedSourceMap = parser.source_map.clone();
                let block = AsyncDependenciesBlock::new(
                  *parser.module_identifier,
                  Into::<DependencyRange>::into(call_expr.span).to_loc(Some(&source_map)),
                  None,
                  vec![dep],
                  Some(alongside_mock_request.to_string()),
                );

                parser.blocks.push(Box::new(block));

                return Some(true);
              } else {
                // add CommonJsRequireDependency
                let mut range_expr: DependencyRange = first_arg.span().into();
                range_expr.end += 1; // TODO: wtf?
                let dep: CommonJsRequireDependency = CommonJsRequireDependency::new(
                  alongside_mock_request.to_string(),
                  range_expr,
                  Some(call_expr.span.into()),
                  parser.in_try,
                  Some(parser.source_map.clone()),
                );

                let range: DependencyRange = call_expr.callee.span().into();
                parser
                  .presentational_dependencies
                  .push(Box::new(RequireHeaderDependency::new(
                    range,
                    Some(parser.source_map.clone()),
                  )));

                parser.dependencies.push(Box::new(dep));
                return Some(true);
              }
            }
          } else {
            return None;
          }
        }

        None
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
  // fn call(
  //   &self,
  //   parser: &mut rspack_plugin_javascript::visitors::JavascriptParser,
  //   call_expr: &CallExpr,
  //   for_name: &str,
  // ) -> Option<bool> {
  //   None
  // }

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
              match (ident.sym.as_str(), prop.sym.as_str()) {
                // rs.mock
                ("rs", "mock") => {
                  self.process_mock(parser, call_expr, true, true);
                  return Some(false);
                }
                // rs.mockRequire
                ("rs", "mockRequire") => {
                  self.process_mock(parser, call_expr, true, false);
                  return Some(false);
                }
                // rs.doMock
                ("rs", "doMock") => {
                  self.process_mock(parser, call_expr, false, true);
                  return Some(false);
                }
                // rs.doMockRequire
                ("rs", "doMockRequire") => {
                  self.process_mock(parser, call_expr, false, false);
                  return Some(false);
                }
                // rs.importActual
                ("rs", "importActual") => {
                  return self.process_import_actual(parser, call_expr);
                }
                // rs.requireActual
                ("rs", "requireActual") => {
                  return self.process_require_actual(parser, call_expr);
                }
                // rs.importMock
                ("rs", "importMock") => {
                  return self.load_mock(parser, call_expr, true);
                }
                // rs.requireMock
                ("rs", "requireMock") => {
                  return self.load_mock(parser, call_expr, false);
                }
                // rs.doUnmock
                ("rs", "doUnmock") => {
                  return self.do_unmock(parser, call_expr, true);
                }
                // rs.resetModules
                ("rs", "resetModules") => {
                  return self.reset_modules(parser, call_expr);
                }
                _ => {
                  // Not a mock module, continue.
                  return None;
                }
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

  // fn member_chain_of_call_member_chain(
  //   &self,
  //   _parser: &mut JavascriptParser,
  //   _expr: &MemberExpr,
  //   _for_name: &str,
  // ) -> Option<bool> {
  //   None
  // }

  // fn evaluate_call_expression_member<'a>(
  //   &self,
  //   _parser: &mut JavascriptParser,
  //   _property: &str,
  //   _expr: &'a CallExpr,
  //   _param: BasicEvaluatedExpression<'a>,
  // ) -> Option<BasicEvaluatedExpression<'a>> {
  //   None
  // }

  // fn import_call(&self, _parser: &mut JavascriptParser, _expr: &CallExpr) -> Option<bool> {
  //   None
  // }

  // fn call_member_chain_of_call_member_chain(
  //   &self,
  //   _parser: &mut JavascriptParser,
  //   _expr: &CallExpr,
  //   _for_name: &str,
  // ) -> Option<bool> {
  //   None
  // }

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
    // if self.hoist_mock_module {
    //   if ident == RS_MOCK {
    //     // self.process_hoist_mock(parser, call_expr);
    //     return None;
    //   } else {
    //     return None;
    //   }
    // }

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
