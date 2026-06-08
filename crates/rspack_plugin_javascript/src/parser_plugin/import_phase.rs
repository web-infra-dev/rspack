use rspack_core::ImportPhase;

use crate::visitors::JavascriptParser;

pub(super) fn get_import_phase(
  parser: &JavascriptParser,
  syntax_phase: swc_core::ecma::ast::ImportPhase,
  webpack_defer: Option<bool>,
  webpack_source: Option<bool>,
) -> ImportPhase {
  let phase_by_syntax = match syntax_phase {
    swc_core::ecma::ast::ImportPhase::Defer
      if parser.javascript_options.defer_import.unwrap_or_default() =>
    {
      ImportPhase::Defer
    }
    swc_core::ecma::ast::ImportPhase::Source
      if parser.javascript_options.source_import.unwrap_or_default() =>
    {
      ImportPhase::Source
    }
    _ => ImportPhase::Evaluation,
  };

  if parser.javascript_options.defer_import.unwrap_or_default()
    && matches!(webpack_defer, Some(true))
  {
    return ImportPhase::Defer;
  }

  if parser.javascript_options.source_import.unwrap_or_default()
    && matches!(webpack_source, Some(true))
  {
    return ImportPhase::Source;
  }

  phase_by_syntax
}
