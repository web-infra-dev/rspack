use std::sync::Arc;

use rspack_core::{DependencyType, collect_ident};
use rspack_plugin_javascript::{
  JavascriptParserPlugin, dependency::ESMCompatibilityDependency, visitors::JavascriptParser,
};

use crate::dependency::ExternalBindingBailout;

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct EsmLibParserPlugin;

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for EsmLibParserPlugin {
  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    // Direct CommonJS external bindings are introduced after parsing. A collision
    // cannot be repaired by the linker's top-level deconfliction, especially
    // inside wrapped modules or function parameters. Inspect decoded AST names
    // so escaped identifiers are covered as well.
    if parser.get_dependencies().iter().any(|dep| {
      matches!(
        dep.dependency_type(),
        DependencyType::CjsRequire
          | DependencyType::CjsFullRequire
          | DependencyType::CjsExportRequire
      )
    }) && collect_ident(parser.ast.allocator, parser.ast.program)
      .iter()
      .any(|ident| ident.id.sym.as_str().starts_with("__rspack_"))
    {
      parser.add_presentational_dependency(Arc::new(ExternalBindingBailout));
    }

    if parser.module_type.is_js_auto()
      && matches!(
        parser.build_meta.exports_type(),
        rspack_core::BuildMetaExportsType::Unset
      )
      && !parser.get_dependencies().iter().any(|dep| {
        matches!(
          dep.dependency_type(),
          DependencyType::CjsExportRequire
            | DependencyType::CjsExports
            | DependencyType::CjsFullRequire
            | DependencyType::CjsRequire
            | DependencyType::CjsSelfReference
            | DependencyType::CommonJSRequireContext
            | DependencyType::ModuleDecorator
        )
      })
    {
      // make module without any exports or module accessing not bail out
      parser
        .build_meta
        .set_exports_type(rspack_core::BuildMetaExportsType::Namespace);
      parser.add_presentational_dependency(Arc::new(ESMCompatibilityDependency));
    }

    None
  }
}

#[cfg(allocative)]
use rspack_util::allocative;
