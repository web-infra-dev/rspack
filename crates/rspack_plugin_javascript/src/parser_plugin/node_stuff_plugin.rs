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

pub struct NodeStuffPlugin;

impl JavascriptParserPlugin for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      unreachable!("ensure only invoke `NodeStuffPlugin` when node options is enabled");
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
    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      unreachable!("ensure only invoke `NodeStuffPlugin` when node options is enabled");
    };
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
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.filename is disabled
        if parser
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
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.dirname is disabled
        if parser
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
        // Skip processing if node.filename is disabled
        if parser
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
        // Skip processing if node.dirname is disabled
        if parser
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
      if parser
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
      if parser
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
      let filename = match parser.compiler_options.node.as_ref().map(|n| n.filename) {
        Some(NodeFilenameOption::False) => return None,
        Some(NodeFilenameOption::Mock) | Some(NodeFilenameOption::WarnMock) => {
          "/index.js".to_string()
        }
        Some(NodeFilenameOption::True) => {
          // Use relative path
          parser
            .resource_data
            .path()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string()
        }
        Some(NodeFilenameOption::EvalOnly) => {
          // For eval-only, use runtime value evaluation
          // The actual replacement is done in member() method
          let resource = parse_resource(parser.resource_data.path()?.as_str())?;
          resource.path.to_string()
        }
        Some(NodeFilenameOption::NodeModule) | None => {
          // Use absolute path for node-module or when no option is set
          Url::from_file_path(parser.resource_data.resource())
            .expect("should be a path")
            .to_file_path()
            .expect("should be a path")
            .to_string_lossy()
            .into_owned()
        }
      };
      Some(eval::evaluate_to_string(filename, start, end))
    } else if for_name == expr_name::IMPORT_META_DIRNAME {
      // Skip processing if importMeta is disabled
      if parser.javascript_options.import_meta == Some(false) {
        return None;
      }
      // Get the appropriate dirname based on node option
      let dirname = match parser.compiler_options.node.as_ref().map(|n| n.dirname) {
        Some(NodeDirnameOption::False) => return None,
        Some(NodeDirnameOption::Mock) | Some(NodeDirnameOption::WarnMock) => "/".to_string(),
        Some(NodeDirnameOption::True) => {
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
        Some(NodeDirnameOption::EvalOnly) => {
          // For eval-only, use runtime value evaluation
          // The actual replacement is done in member() method
          parser.resource_data.path()?.parent()?.to_string()
        }
        Some(NodeDirnameOption::NodeModule) | None => {
          // Use absolute path for node-module or when no option is set
          Url::from_file_path(parser.resource_data.resource())
            .expect("should be a path")
            .to_file_path()
            .expect("should be a path")
            .parent()
            .expect("should have a parent")
            .to_string_lossy()
            .into_owned()
        }
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
        let replacement = match parser.compiler_options.node.as_ref().map(|n| n.filename) {
          Some(NodeFilenameOption::False) => return None,
          Some(NodeFilenameOption::Mock) => "'/index.js'".to_string(),
          Some(NodeFilenameOption::WarnMock) => {
            parser.add_warning(Diagnostic::warn(
              "NODE_IMPORT_META_FILENAME".to_string(),
              "\"import.meta.filename\" has been used, it will be mocked.".to_string(),
            ));
            "'/index.js'".to_string()
          }
          Some(NodeFilenameOption::True) => {
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
          Some(NodeFilenameOption::EvalOnly) => {
            // For ESM output, keep as import.meta.filename
            if parser.compiler_options.output.module {
              "import.meta.filename".to_string()
            } else {
              // For CJS output, replace with __filename
              "__filename".to_string()
            }
          }
          Some(NodeFilenameOption::NodeModule) | None => {
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
        let replacement = match parser.compiler_options.node.as_ref().map(|n| n.dirname) {
          Some(NodeDirnameOption::False) => return None,
          Some(NodeDirnameOption::Mock) => "'/'".to_string(),
          Some(NodeDirnameOption::WarnMock) => {
            parser.add_warning(Diagnostic::warn(
              "NODE_IMPORT_META_DIRNAME".to_string(),
              "\"import.meta.dirname\" has been used, it will be mocked.".to_string(),
            ));
            "'/'".to_string()
          }
          Some(NodeDirnameOption::True) => {
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
          Some(NodeDirnameOption::EvalOnly) => {
            // For ESM output, keep as import.meta.dirname
            if parser.compiler_options.output.module {
              "import.meta.dirname".to_string()
            } else {
              // For CJS output, replace with __dirname
              "__dirname".to_string()
            }
          }
          Some(NodeDirnameOption::NodeModule) | None => {
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
        let value = match parser.compiler_options.node.as_ref().map(|n| n.filename) {
          Some(NodeFilenameOption::False) => return None,
          Some(NodeFilenameOption::Mock) => "\"/index.js\"".to_string(),
          Some(NodeFilenameOption::WarnMock) => {
            parser.add_warning(Diagnostic::warn(
              "NODE_IMPORT_META_FILENAME".to_string(),
              "\"import.meta.filename\" has been used, it will be mocked.".to_string(),
            ));
            "\"/index.js\"".to_string()
          }
          Some(NodeFilenameOption::True) => {
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
          Some(NodeFilenameOption::EvalOnly) => {
            // For ESM output, keep as import.meta.filename
            if parser.compiler_options.output.module {
              "import.meta.filename".to_string()
            } else {
              // For CJS output, replace with __filename
              "__filename".to_string()
            }
          }
          Some(NodeFilenameOption::NodeModule) | None => {
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
        };
        Some(format!(r#"filename: {value}"#))
      }
      "dirname" => {
        // Check node.dirname option and get the appropriate value
        let value = match parser.compiler_options.node.as_ref().map(|n| n.dirname) {
          Some(NodeDirnameOption::False) => return None,
          Some(NodeDirnameOption::Mock) => "\"/\"".to_string(),
          Some(NodeDirnameOption::WarnMock) => {
            parser.add_warning(Diagnostic::warn(
              "NODE_IMPORT_META_DIRNAME".to_string(),
              "\"import.meta.dirname\" has been used, it will be mocked.".to_string(),
            ));
            "\"/\"".to_string()
          }
          Some(NodeDirnameOption::True) => {
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
          Some(NodeDirnameOption::EvalOnly) => {
            // For ESM output, keep as import.meta.dirname
            if parser.compiler_options.output.module {
              "import.meta.dirname".to_string()
            } else {
              // For CJS output, replace with __dirname
              "__dirname".to_string()
            }
          }
          Some(NodeDirnameOption::NodeModule) | None => {
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
        };
        Some(format!(r#"dirname: {value}"#))
      }
      _ => None,
    }
  }
}
