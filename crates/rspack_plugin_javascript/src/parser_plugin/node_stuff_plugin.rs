use rspack_core::{
  CachedConstDependency, ConstDependency, NodeDirnameOption, NodeFilenameOption, NodeGlobalOption,
  RuntimeGlobals, get_context, parse_resource,
};
use sugar_path::SugarPath;
use swc_core::{common::Spanned, ecma::ast::Expr};

use super::JavascriptParserPlugin;
use crate::{dependency::ExternalModuleDependency, utils::eval, visitors::JavascriptParser};

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
        NodeDirnameOption::WarnMock => Some("/".to_string()),
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
        _ => None,
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
        NodeFilenameOption::WarnMock => Some("/index.js".to_string()),
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
        NodeFilenameOption::True => Some(
          parser
            .resource_data
            .path()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        _ => None,
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
        parser
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::GLOBAL)
          .into(),
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
        parser
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::GLOBAL)
          .into(),
        Some(RuntimeGlobals::GLOBAL),
      )));
      return Some(false);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
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
    } else {
      None
    }
  }
}
