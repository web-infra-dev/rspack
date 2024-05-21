use rustc_hash::FxHashSet;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct JavascriptMetaInfoPlugin;

impl JavascriptParserPlugin for JavascriptMetaInfoPlugin {
  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    if parser.build_info.top_level_declarations.is_none() {
      parser.build_info.top_level_declarations = Some(FxHashSet::default());
    }
    let variables: Vec<_> = parser
      .get_all_variables_from_current_scope()
      .map(|(name, _)| name.to_string())
      .collect();
    for name in variables {
      if parser.get_free_info_from_variable(&name).is_none() {
        parser
          .build_info
          .top_level_declarations
          .as_mut()
          .expect("must have value")
          .insert(name);
      }
    }
    None
  }
}
