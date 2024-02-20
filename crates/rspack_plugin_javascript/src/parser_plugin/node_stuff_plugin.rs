use rspack_core::{get_context, ConstDependency, RuntimeGlobals, SpanExt};
use sugar_path::SugarPath;

use super::JavascriptParserPlugin;
use crate::utils::eval;

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
      let dirname = match node_option.dirname.as_str() {
        "mock" => Some("/".to_string()),
        "warn-mock" => Some("/".to_string()),
        "true" => Some(
          parser
            .resource_data
            .resource_path
            .parent()
            .expect("TODO:")
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
            format!("'{dirname}'").into(),
            None,
          )));
        return Some(true);
      }
    } else if str == FILE_NAME {
      let filename = match node_option.filename.as_str() {
        "mock" => Some("/index.js".to_string()),
        "warn-mock" => Some("/index.js".to_string()),
        "true" => Some(
          parser
            .resource_data
            .resource_path
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
    } else if str == GLOBAL && matches!(node_option.global.as_str(), "true" | "warn") {
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
