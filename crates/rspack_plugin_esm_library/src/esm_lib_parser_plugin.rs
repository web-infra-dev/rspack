use rspack_core::DependencyType;
use rspack_plugin_javascript::{
  JavascriptParserPlugin, dependency::ESMCompatibilityDependency, visitors::JavascriptParser,
};
pub struct EsmLibParserPlugin;

impl JavascriptParserPlugin for EsmLibParserPlugin {
  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    if parser.module_type.is_js_auto()
      && matches!(
        parser.build_meta.exports_type,
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
        )
      })
    {
      parser.build_meta.exports_type = rspack_core::BuildMetaExportsType::Namespace;
      parser.add_presentational_dependency(Box::new(ESMCompatibilityDependency));
    }

    None
  }
}
