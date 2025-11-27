use rspack_core::{
  CachedConstDependency, ConstDependency, NodeDirnameOption, NodeFilenameOption, NodeGlobalOption,
  RuntimeGlobals, get_context, parse_resource,
};
use rspack_error::Diagnostic;
use rspack_util::SpanExt;
use sugar_path::SugarPath;
use swc_core::{common::Spanned, ecma::ast::Expr};
use url::Url;

use crate::{
  JavascriptParserPlugin,
  dependency::ExternalModuleDependency,
  utils::eval,
  visitors::{DestructuringAssignmentProperty, JavascriptParser},
};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const GLOBAL: &str = "global";

/// Plugin for handling Node.js-specific variables like `__dirname`, `__filename`, `global`,
/// and their ESM equivalents `import.meta.dirname` and `import.meta.filename`.
///
/// When `esm_only` is true, this plugin only handles `import.meta.dirname/filename`
/// (used for ESM modules). When false, it handles `__dirname/__filename/global` as well
/// (used for CJS modules).
pub struct NodeStuffPlugin {
  /// When true, only handle import.meta.dirname/filename (for ESM modules).
  /// When false, also handle __dirname/__filename/global (for CJS modules).
  esm_only: bool,
}

impl NodeStuffPlugin {
  pub fn new(esm_only: bool) -> Self {
    Self { esm_only }
  }
}

impl JavascriptParserPlugin for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    // For ESM-only mode, skip __dirname/__filename/global handling
    if self.esm_only {
      return None;
    }

    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      // When node: false, this plugin is not registered for CJS modules
      return None;
    };
    if for_name == DIR_NAME {
      let dirname = match node_option.dirname {
        NodeDirnameOption::Mock => Some("/".to_string()),
        NodeDirnameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_DIRNAME".to_string(),
            format!("\"{DIR_NAME}\" has been used, it will be mocked."),
          ));
          Some("/".to_string())
        }
        NodeDirnameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          let external_url_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let external_path_dep = ExternalModuleDependency::new(
            "path".to_string(),
            vec![("dirname".to_string(), "__webpack_dirname__".to_string())],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            DIR_NAME.into(),
            "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_url_dep));
          parser.add_presentational_dependency(Box::new(external_path_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
          return Some(true);
        }
        NodeDirnameOption::EvalOnly => {
          // For CJS output, preserve __dirname (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }

          let external_url_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let external_path_dep = ExternalModuleDependency::new(
            "path".to_string(),
            vec![("dirname".to_string(), "__webpack_dirname__".to_string())],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            DIR_NAME.into(),
            "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_url_dep));
          parser.add_presentational_dependency(Box::new(external_path_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
          return Some(true);
        }
        NodeDirnameOption::True => Some(
          parser
            .resource_data
            .path()?
            .parent()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        NodeDirnameOption::False => None,
      };
      if let Some(dirname) = dirname {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&dirname)
            .expect("should render dirname")
            .into(),
          None,
        )));
        return Some(true);
      }
    } else if for_name == FILE_NAME {
      let filename = match node_option.filename {
        NodeFilenameOption::Mock => Some("/index.js".to_string()),
        NodeFilenameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_FILENAME".to_string(),
            format!("\"{FILE_NAME}\" has been used, it will be mocked."),
          ));
          Some("/index.js".to_string())
        }
        NodeFilenameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          let external_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            FILE_NAME.into(),
            "__webpack_fileURLToPath__(import.meta.url)"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
          return Some(true);
        }
        NodeFilenameOption::EvalOnly => {
          // For CJS output, preserve __filename (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }
          let external_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            FILE_NAME.into(),
            "__webpack_fileURLToPath__(import.meta.url)"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
          return Some(true);
        }
        NodeFilenameOption::True => Some(
          parser
            .resource_data
            .path()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        NodeFilenameOption::False => None,
      };
      if let Some(filename) = filename {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&filename)
            .expect("should render filename")
            .into(),
          None,
        )));
        return Some(true);
      }
    } else if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        ident.span.into(),
        RuntimeGlobals::GLOBAL.name().into(),
        Some(RuntimeGlobals::GLOBAL),
      )));
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    // For ESM-only mode, skip global handling
    if self.esm_only {
      return None;
    }

    let node_option = parser.compiler_options.node.as_ref()?;
    if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span().into(),
        RuntimeGlobals::GLOBAL.name().into(),
        Some(RuntimeGlobals::GLOBAL),
      )));
      return Some(false);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    use crate::visitors::expr_name;

    match for_name {
      FILE_NAME | expr_name::IMPORT_META_FILENAME => {
        // Skip __filename when in esm_only mode (only handle import.meta.filename)
        if self.esm_only && for_name == FILE_NAME {
          return None;
        }
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node: false or node.filename is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        // fallthrough
      }
      DIR_NAME | expr_name::IMPORT_META_DIRNAME => {
        // Skip __dirname when in esm_only mode (only handle import.meta.dirname)
        if self.esm_only && for_name == DIR_NAME {
          return None;
        }
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node: false or node.dirname is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        // fallthrough
      }
      _ => return None,
    }

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      "'string'".into(),
      None,
    )));
    Some(true)
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    use crate::visitors::expr_name;

    match for_name {
      expr_name::IMPORT_META_FILENAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node: false or node.filename is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        Some(eval::evaluate_to_string(
          "string".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        ))
      }
      expr_name::IMPORT_META_DIRNAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node: false or node.dirname is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        Some(eval::evaluate_to_string(
          "string".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        ))
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    use crate::visitors::expr_name;

    if for_name == DIR_NAME {
      // Skip if node: false or node.dirname is disabled
      if parser.compiler_options.node.is_none()
        || parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
      {
        return None;
      }
      Some(eval::evaluate_to_string(
        get_context(parser.resource_data).as_str().to_string(),
        start,
        end,
      ))
    } else if for_name == FILE_NAME {
      // Skip if node: false or node.filename is disabled
      if parser.compiler_options.node.is_none()
        || parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
      {
        return None;
      }
      let resource = parse_resource(parser.resource_data.path()?.as_str())?;
      Some(eval::evaluate_to_string(
        resource.path.to_string(),
        start,
        end,
      ))
    } else if for_name == expr_name::IMPORT_META_FILENAME {
      // Skip processing if importMeta is disabled
      if parser.javascript_options.import_meta == Some(false) {
        return None;
      }
      // Get the appropriate filename based on node option
      let filename = match parser.compiler_options.node.as_ref() {
        // When node: false, don't evaluate - preserve as runtime value
        None => return None,
        Some(node_option) => match node_option.filename {
          NodeFilenameOption::False => return None,
          NodeFilenameOption::Mock | NodeFilenameOption::WarnMock => "/index.js".to_string(),
          NodeFilenameOption::True => {
            // Use relative path
            parser
              .resource_data
              .path()?
              .as_std_path()
              .relative(&parser.compiler_options.context)
              .to_string_lossy()
              .to_string()
          }
          NodeFilenameOption::EvalOnly => {
            // For eval-only, use runtime value evaluation
            // The actual replacement is done in member() method
            let resource = parse_resource(parser.resource_data.path()?.as_str())?;
            resource.path.to_string()
          }
          NodeFilenameOption::NodeModule => {
            // Use absolute path for node-module
            Url::from_file_path(parser.resource_data.resource())
              .expect("should be a path")
              .to_file_path()
              .expect("should be a path")
              .to_string_lossy()
              .into_owned()
          }
        },
      };
      Some(eval::evaluate_to_string(filename, start, end))
    } else if for_name == expr_name::IMPORT_META_DIRNAME {
      // Skip processing if importMeta is disabled
      if parser.javascript_options.import_meta == Some(false) {
        return None;
      }
      // Get the appropriate dirname based on node option
      let dirname = match parser.compiler_options.node.as_ref() {
        // When node: false, don't evaluate - preserve as runtime value
        None => return None,
        Some(node_option) => match node_option.dirname {
          NodeDirnameOption::False => return None,
          NodeDirnameOption::Mock | NodeDirnameOption::WarnMock => "/".to_string(),
          NodeDirnameOption::True => {
            // Use relative path
            parser
              .resource_data
              .path()?
              .parent()?
              .as_std_path()
              .relative(&parser.compiler_options.context)
              .to_string_lossy()
              .to_string()
          }
          NodeDirnameOption::EvalOnly => {
            // For eval-only, use runtime value evaluation
            // The actual replacement is done in member() method
            parser.resource_data.path()?.parent()?.to_string()
          }
          NodeDirnameOption::NodeModule => {
            // Use absolute path for node-module
            Url::from_file_path(parser.resource_data.resource())
              .expect("should be a path")
              .to_file_path()
              .expect("should be a path")
              .parent()
              .expect("should have a parent")
              .to_string_lossy()
              .into_owned()
          }
        },
      };
      Some(eval::evaluate_to_string(dirname, start, end))
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
    use crate::visitors::expr_name;

    match for_name {
      expr_name::IMPORT_META_FILENAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Check node.filename option and get the appropriate value
        let replacement = match parser.compiler_options.node.as_ref() {
          // When node: false, preserve import.meta.filename as-is
          None => "import.meta.filename".to_string(),
          Some(node_option) => match node_option.filename {
            NodeFilenameOption::False => "import.meta.filename".to_string(),
            NodeFilenameOption::Mock => "'/index.js'".to_string(),
            NodeFilenameOption::WarnMock => {
              parser.add_warning(Diagnostic::warn(
                "NODE_IMPORT_META_FILENAME".to_string(),
                "\"import.meta.filename\" has been used, it will be mocked.".to_string(),
              ));
              "'/index.js'".to_string()
            }
            NodeFilenameOption::True => {
              // Use relative path
              let filename = parser
                .resource_data
                .path()?
                .as_std_path()
                .relative(&parser.compiler_options.context)
                .to_string_lossy()
                .to_string();
              format!("'{filename}'")
            }
            NodeFilenameOption::EvalOnly => {
              // For ESM output, keep as import.meta.filename
              if parser.compiler_options.output.module {
                "import.meta.filename".to_string()
              } else {
                // For CJS output, replace with __filename
                "__filename".to_string()
              }
            }
            NodeFilenameOption::NodeModule => {
              // For node-module mode, check if native import.meta.filename is supported
              if parser.compiler_options.output.module
                && parser
                  .compiler_options
                  .output
                  .environment
                  .import_meta_dirname_and_filename
                  .unwrap_or(false)
              {
                // Keep as import.meta.filename - runtime supports it
                return None;
              }
              // Use runtime expression with import.meta.url
              let external_dep = ExternalModuleDependency::new(
                "url".to_string(),
                vec![(
                  "fileURLToPath".to_string(),
                  "__webpack_fileURLToPath__".to_string(),
                )],
                None,
              );
              parser.add_presentational_dependency(Box::new(external_dep));
              "__webpack_fileURLToPath__(import.meta.url)".to_string()
            }
          },
        };
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          replacement.into(),
          None,
        )));
        Some(true)
      }
      expr_name::IMPORT_META_DIRNAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Check node.dirname option and get the appropriate value
        let replacement = match parser.compiler_options.node.as_ref() {
          // When node: false, preserve import.meta.dirname as-is
          None => "import.meta.dirname".to_string(),
          Some(node_option) => match node_option.dirname {
            NodeDirnameOption::False => "import.meta.dirname".to_string(),
            NodeDirnameOption::Mock => "'/'".to_string(),
            NodeDirnameOption::WarnMock => {
              parser.add_warning(Diagnostic::warn(
                "NODE_IMPORT_META_DIRNAME".to_string(),
                "\"import.meta.dirname\" has been used, it will be mocked.".to_string(),
              ));
              "'/'".to_string()
            }
            NodeDirnameOption::True => {
              // Use relative path
              let dirname = parser
                .resource_data
                .path()?
                .parent()?
                .as_std_path()
                .relative(&parser.compiler_options.context)
                .to_string_lossy()
                .to_string();
              format!("'{dirname}'")
            }
            NodeDirnameOption::EvalOnly => {
              // For ESM output, keep as import.meta.dirname
              if parser.compiler_options.output.module {
                "import.meta.dirname".to_string()
              } else {
                // For CJS output, replace with __dirname
                "__dirname".to_string()
              }
            }
            NodeDirnameOption::NodeModule => {
              // For node-module mode, check if native import.meta.dirname is supported
              if parser.compiler_options.output.module
                && parser
                  .compiler_options
                  .output
                  .environment
                  .import_meta_dirname_and_filename
                  .unwrap_or(false)
              {
                // Keep as import.meta.dirname - runtime supports it
                return None;
              }
              // Use runtime expression with import.meta.url
              let external_url_dep = ExternalModuleDependency::new(
                "url".to_string(),
                vec![(
                  "fileURLToPath".to_string(),
                  "__webpack_fileURLToPath__".to_string(),
                )],
                None,
              );
              let external_path_dep = ExternalModuleDependency::new(
                "path".to_string(),
                vec![("dirname".to_string(), "__webpack_dirname__".to_string())],
                None,
              );
              parser.add_presentational_dependency(Box::new(external_url_dep));
              parser.add_presentational_dependency(Box::new(external_path_dep));
              "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))".to_string()
            }
          },
        };
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          replacement.into(),
          None,
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn import_meta_property_in_destructuring(
    &self,
    parser: &mut JavascriptParser,
    property: &DestructuringAssignmentProperty,
  ) -> Option<String> {
    // Skip processing if importMeta is disabled
    if parser.javascript_options.import_meta == Some(false) {
      return None;
    }
    match property.id.as_str() {
      "filename" => {
        // Check node.filename option and get the appropriate value
        let value = match parser.compiler_options.node.as_ref() {
          // When node: false, preserve import.meta.filename as-is
          None => "import.meta.filename".to_string(),
          Some(node_option) => match node_option.filename {
            // Keep `import.meta.filename` in code when node.__filename is false
            NodeFilenameOption::False => "import.meta.filename".to_string(),
            NodeFilenameOption::Mock => "\"/index.js\"".to_string(),
            NodeFilenameOption::WarnMock => {
              parser.add_warning(Diagnostic::warn(
                "NODE_IMPORT_META_FILENAME".to_string(),
                "\"import.meta.filename\" has been used, it will be mocked.".to_string(),
              ));
              "\"/index.js\"".to_string()
            }
            NodeFilenameOption::True => {
              // Use relative path
              let filename = parser
                .resource_data
                .path()?
                .as_std_path()
                .relative(&parser.compiler_options.context)
                .to_string_lossy()
                .to_string();
              format!(r#""{filename}""#)
            }
            NodeFilenameOption::EvalOnly => {
              // For ESM output, keep as import.meta.filename
              if parser.compiler_options.output.module {
                "import.meta.filename".to_string()
              } else {
                // For CJS output, replace with __filename
                "__filename".to_string()
              }
            }
            NodeFilenameOption::NodeModule => {
              // For node-module mode, check if native import.meta.filename is supported
              if parser.compiler_options.output.module
                && parser
                  .compiler_options
                  .output
                  .environment
                  .import_meta_dirname_and_filename
                  .unwrap_or(false)
              {
                // Keep as import.meta.filename - runtime supports it
                "import.meta.filename".to_string()
              } else {
                // Use runtime expression with import.meta.url
                let external_dep = ExternalModuleDependency::new(
                  "url".to_string(),
                  vec![(
                    "fileURLToPath".to_string(),
                    "__webpack_fileURLToPath__".to_string(),
                  )],
                  None,
                );
                parser.add_presentational_dependency(Box::new(external_dep));
                "__webpack_fileURLToPath__(import.meta.url)".to_string()
              }
            }
          },
        };
        Some(format!(r#"filename: {value}"#))
      }
      "dirname" => {
        // Check node.dirname option and get the appropriate value
        let value = match parser.compiler_options.node.as_ref() {
          // When node: false, preserve import.meta.dirname as-is
          None => "import.meta.dirname".to_string(),
          Some(node_option) => match node_option.dirname {
            NodeDirnameOption::False => "import.meta.dirname".to_string(),
            NodeDirnameOption::Mock => "\"/\"".to_string(),
            NodeDirnameOption::WarnMock => {
              parser.add_warning(Diagnostic::warn(
                "NODE_IMPORT_META_DIRNAME".to_string(),
                "\"import.meta.dirname\" has been used, it will be mocked.".to_string(),
              ));
              "\"/\"".to_string()
            }
            NodeDirnameOption::True => {
              // Use relative path
              let dirname = parser
                .resource_data
                .path()?
                .parent()?
                .as_std_path()
                .relative(&parser.compiler_options.context)
                .to_string_lossy()
                .to_string();
              format!(r#""{dirname}""#)
            }
            NodeDirnameOption::EvalOnly => {
              // For ESM output, keep as import.meta.dirname
              if parser.compiler_options.output.module {
                "import.meta.dirname".to_string()
              } else {
                // For CJS output, replace with __dirname
                "__dirname".to_string()
              }
            }
            NodeDirnameOption::NodeModule => {
              // For node-module mode, check if native import.meta.dirname is supported
              if parser.compiler_options.output.module
                && parser
                  .compiler_options
                  .output
                  .environment
                  .import_meta_dirname_and_filename
                  .unwrap_or(false)
              {
                // Keep as import.meta.dirname - runtime supports it
                "import.meta.dirname".to_string()
              } else {
                // Use runtime expression with import.meta.url
                let external_url_dep = ExternalModuleDependency::new(
                  "url".to_string(),
                  vec![(
                    "fileURLToPath".to_string(),
                    "__webpack_fileURLToPath__".to_string(),
                  )],
                  None,
                );
                let external_path_dep = ExternalModuleDependency::new(
                  "path".to_string(),
                  vec![("dirname".to_string(), "__webpack_dirname__".to_string())],
                  None,
                );
                parser.add_presentational_dependency(Box::new(external_url_dep));
                parser.add_presentational_dependency(Box::new(external_path_dep));
                "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))".to_string()
              }
            }
          },
        };
        Some(format!(r#"dirname: {value}"#))
      }
      _ => None,
    }
  }
}
