use rspack_core::{
  CachedConstDependency, CachedConstDependencyPlace, ConstDependency, NodeDirnameOption,
  NodeFilenameOption, NodeGlobalOption, RuntimeGlobals, RuntimeRequirementsDependency, get_context,
  parse_resource,
};
use rspack_error::{Diagnostic, cyan, yellow};
use sugar_path::SugarPath;
use swc_experimental_ecma_ast::{Expr, GetSpan, Ident, UnaryExpr};

use crate::{
  JavascriptParserPlugin, dependency::ExternalModuleDependency, utils::eval,
  visitors::JavascriptParser,
};

const DIRNAME: &str = "__dirname";
const FILENAME: &str = "__filename";
const GLOBAL: &str = "global";
const MOCK_DIRNAME: &str = "/";
const MOCK_FILENAME: &str = "/index.js";

#[derive(Clone, Copy)]
enum NodeMetaProperty {
  Filename,
  Dirname,
}

impl NodeMetaProperty {
  fn import_meta_cached_identifier(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "__rspack_import_meta_filename__",
      NodeMetaProperty::Dirname => "__rspack_import_meta_dirname__",
    }
  }

  fn node_module_runtime_expr(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "__rspack_fileURLToPath(import.meta.url)",
      NodeMetaProperty::Dirname => "__rspack_dirname(__rspack_fileURLToPath(import.meta.url))",
    }
  }
}

/// Plugin for handling Node.js-specific variables like `__dirname`, `__filename`, and `global`.
///
/// This mirrors webpack's approach where NodeStuffPlugin is registered once per module type
/// with boolean flags controlling which features to handle.
pub struct NodeStuffPlugin {
  /// When true, handle __dirname/__filename/global (CJS features)
  handle_cjs: bool,
}

impl NodeStuffPlugin {
  pub fn new(handle_cjs: bool) -> Self {
    Self { handle_cjs }
  }

  fn get_relative_path(parser: &JavascriptParser, property: NodeMetaProperty) -> Option<String> {
    match property {
      NodeMetaProperty::Filename => Some(
        parser
          .resource_data
          .path()?
          .as_std_path()
          .relative(&parser.compiler_options.context)
          .to_string_lossy()
          .to_string(),
      ),
      NodeMetaProperty::Dirname => Some(
        parser
          .resource_data
          .path()?
          .parent()?
          .as_std_path()
          .relative(&parser.compiler_options.context)
          .to_string_lossy()
          .to_string(),
      ),
    }
  }

  fn add_node_module_dependencies(parser: &mut JavascriptParser, property: NodeMetaProperty) {
    let external_url_dep = ExternalModuleDependency::new(
      "url".to_string(),
      vec![(
        "fileURLToPath".to_string(),
        "__rspack_fileURLToPath".to_string(),
      )],
      None,
    );
    parser.add_presentational_dependency(Box::new(external_url_dep));

    if matches!(property, NodeMetaProperty::Dirname) {
      let external_path_dep = ExternalModuleDependency::new(
        "path".to_string(),
        vec![("dirname".to_string(), "__rspack_dirname".to_string())],
        None,
      );
      parser.add_presentational_dependency(Box::new(external_path_dep));
    }
  }

  fn add_cjs_node_module_dependency(
    parser: &mut JavascriptParser,
    ident_span: swc_experimental_ecma_ast::Span,
    name: &str,
    property: NodeMetaProperty,
  ) {
    Self::add_node_module_dependencies(parser, property);
    let place = if parser.compiler_options.output.module {
      CachedConstDependencyPlace::Chunk
    } else {
      CachedConstDependencyPlace::Module
    };
    let identifier = if matches!(place, CachedConstDependencyPlace::Chunk) {
      property.import_meta_cached_identifier()
    } else {
      name
    };
    let const_dep = CachedConstDependency::new_with_place(
      ident_span.into(),
      identifier.into(),
      property.node_module_runtime_expr().into(),
      place,
    );
    parser.add_presentational_dependency(Box::new(const_dep));
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    // Skip CJS handling if not enabled
    if !self.handle_cjs {
      return None;
    }

    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      // When node: false, this plugin is not registered for CJS modules
      return None;
    };
    if for_name == DIRNAME {
      let dirname = match node_option.dirname {
        NodeDirnameOption::Mock => Some(MOCK_DIRNAME.to_string()),
        NodeDirnameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_DIRNAME".to_string(),
            format!("\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.", yellow(&DIRNAME), cyan(&"node.__dirname")),
          ));
          Some(MOCK_DIRNAME.to_string())
        }
        NodeDirnameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            DIRNAME,
            NodeMetaProperty::Dirname,
          );
          return Some(true);
        }
        NodeDirnameOption::EvalOnly => {
          // For CJS output, preserve __dirname (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            DIRNAME,
            NodeMetaProperty::Dirname,
          );
          return Some(true);
        }
        NodeDirnameOption::True => Self::get_relative_path(parser, NodeMetaProperty::Dirname),
        NodeDirnameOption::False => None,
      };
      if let Some(dirname) = dirname {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          rspack_util::json_stringify_str(&dirname).into(),
        )));
        return Some(true);
      }
    } else if for_name == FILENAME {
      let filename = match node_option.filename {
        NodeFilenameOption::Mock => Some(MOCK_FILENAME.to_string()),
        NodeFilenameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_FILENAME".to_string(),
            format!("\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.", yellow(&FILENAME), cyan(&"node.__filename")),
          ));
          Some(MOCK_FILENAME.to_string())
        }
        NodeFilenameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            FILENAME,
            NodeMetaProperty::Filename,
          );
          return Some(true);
        }
        NodeFilenameOption::EvalOnly => {
          // For CJS output, preserve __filename (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            FILENAME,
            NodeMetaProperty::Filename,
          );
          return Some(true);
        }
        NodeFilenameOption::True => Self::get_relative_path(parser, NodeMetaProperty::Filename),
        NodeFilenameOption::False => None,
      };
      if let Some(filename) = filename {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          rspack_util::json_stringify_str(&filename).into(),
        )));
        return Some(true);
      }
    } else if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        ident.span.into(),
        RuntimeGlobals::GLOBAL,
      )));
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser<'p>, expr: &Expr, for_name: &str) -> Option<bool> {
    // Skip CJS handling if not enabled
    if !self.handle_cjs {
      return None;
    }

    let node_option = parser.compiler_options.node.as_ref()?;
    if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        expr.span().into(),
        RuntimeGlobals::GLOBAL,
      )));
      return Some(false);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    unary_expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      FILENAME => {
        // Skip CJS __filename if not handling CJS
        if !self.handle_cjs {
          return None;
        }
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
      }
      DIRNAME => {
        // Skip CJS __dirname if not handling CJS
        if !self.handle_cjs {
          return None;
        }
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
      }
      _ => return None,
    }

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      "'string'".into(),
    )));
    Some(true)
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
    if for_name == DIRNAME {
      // Skip CJS handling if not enabled
      if !self.handle_cjs {
        return None;
      }
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
    } else if for_name == FILENAME {
      // Skip CJS handling if not enabled
      if !self.handle_cjs {
        return None;
      }
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
    } else {
      None
    }
  }
}
