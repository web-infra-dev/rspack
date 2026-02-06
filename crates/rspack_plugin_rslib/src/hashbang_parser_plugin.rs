use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_core::ecma::ast::Program;

pub(crate) struct HashbangParserPlugin;

impl JavascriptParserPlugin for HashbangParserPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    let hashbang = ast
      .as_module()
      .and_then(|m| m.shebang.as_ref())
      .or_else(|| ast.as_script().and_then(|s| s.shebang.as_ref()))?;

    // Normalize hashbang to always include "#!" prefix
    // SWC may omit the leading "#!" in the shebang value
    let normalized_hashbang = if hashbang.starts_with("#!") {
      hashbang.to_string()
    } else {
      format!("#!{hashbang}")
    };

    // Store hashbang in build_info for later use during rendering
    parser.build_info.extras.insert(
      "hashbang".to_string(),
      serde_json::Value::String(normalized_hashbang),
    );

    // Remove hashbang from source code
    // If SWC omitted "#!", we still need to remove those two characters
    let replace_len = if hashbang.starts_with("#!") {
      hashbang.len() as u32
    } else {
      hashbang.len() as u32 + 2 // include "#!"
    };

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (0, replace_len).into(),
      "".into(),
    )));

    None
  }
}
