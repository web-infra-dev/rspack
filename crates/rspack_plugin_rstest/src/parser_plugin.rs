use rspack_plugin_javascript::JavascriptParserPlugin;

use crate::module_path_name_dependency::{ModulePathNameDependency, NameType};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";

#[derive(Debug, Default)]
pub struct RstestParserPlugin;

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
}
