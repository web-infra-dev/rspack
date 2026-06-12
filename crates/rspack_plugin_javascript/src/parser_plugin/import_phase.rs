use rspack_core::ImportPhase;
use swc_experimental_ecma_ast::ImportPhase as AstImportPhase;

use crate::visitors::JavascriptParser;

pub(super) fn get_import_phase(
  parser: &JavascriptParser,
  syntax_phase: AstImportPhase,
) -> ImportPhase {
  match syntax_phase {
    AstImportPhase::Defer if parser.javascript_options.defer_import.unwrap_or_default() => {
      ImportPhase::Defer
    }
    AstImportPhase::Source if parser.javascript_options.source_import.unwrap_or_default() => {
      ImportPhase::Source
    }
    _ => ImportPhase::Evaluation,
  }
}
