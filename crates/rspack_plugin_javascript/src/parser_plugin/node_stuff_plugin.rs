use rspack_core::{
  get_context, CachedConstDependency, ConstDependency, NodeDirnameOption, NodeFilenameOption,
  NodeGlobalOption, RuntimeGlobals, SpanExt,
};
use sugar_path::SugarPath;

use super::JavascriptParserPlugin;
use crate::{dependency::ExternalModuleDependency, utils::eval};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const GLOBAL: &str = "global";

pub struct NodeStuffPlugin;

impl JavascriptParserPlugin for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    _for_name: &str,
  ) -> Option<bool> {
    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      unreachable!("ensure only invoke `NodeStuffPlugin` when node options is enabled");
    };
    let str = ident.sym.as_str();
    if !parser.is_unresolved_ident(str) {
      return None;
    }
    if str == DIR_NAME {
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
            ident.span.real_lo(),
            ident.span.real_hi(),
            DIR_NAME.into(),
            "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))"
              .to_string()
              .into(),
          );

          parser
            .presentational_dependencies
            .push(Box::new(external_url_dep));
          parser
            .presentational_dependencies
            .push(Box::new(external_path_dep));
          parser.presentational_dependencies.push(Box::new(const_dep));
          return Some(true);
        }
        NodeDirnameOption::True => Some(
          parser
            .resource_data
            .resource_path
            .as_deref()?
            .parent()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        _ => None,
      };
      if let Some(dirname) = dirname {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            serde_json::to_string(&dirname)
              .expect("should render dirname")
              .into(),
            None,
          )));
        return Some(true);
      }
    } else if str == FILE_NAME {
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
            ident.span.real_lo(),
            ident.span.real_hi(),
            FILE_NAME.into(),
            "__webpack_fileURLToPath__(import.meta.url)"
              .to_string()
              .into(),
          );

          parser
            .presentational_dependencies
            .push(Box::new(external_dep));
          parser.presentational_dependencies.push(Box::new(const_dep));
          return Some(true);
        }
        NodeFilenameOption::True => Some(
          parser
            .resource_data
            .resource_path
            .as_deref()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        _ => None,
      };
      if let Some(filename) = filename {
        parser
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            serde_json::to_string(&filename)
              .expect("should render filename")
              .into(),
            None,
          )));
        return Some(true);
      }
    } else if str == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          ident.span.real_lo(),
          ident.span.real_hi(),
          RuntimeGlobals::GLOBAL.name().into(),
          Some(RuntimeGlobals::GLOBAL),
        )));
      return Some(true);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if ident == DIR_NAME {
      Some(eval::evaluate_to_string(
        get_context(parser.resource_data).as_str().to_string(),
        start,
        end,
      ))
    } else {
      None
    }
  }
}
