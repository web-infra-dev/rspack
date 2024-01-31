use rspack_core::needs_refactor::WorkerSyntaxList;
use rspack_core::needs_refactor::{collect_from_import_decl, init_worker_syntax_scanner};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct WorkerSyntaxScanner {
  caps: Vec<(&'static str, &'static str)>,
}

impl WorkerSyntaxScanner {
  pub fn new(syntax: &'static [&'static str], list: &mut WorkerSyntaxList) -> Self {
    let mut caps = Vec::new();
    init_worker_syntax_scanner(syntax, &mut caps, list);
    Self { caps }
  }
}

impl JavascriptParserPlugin for WorkerSyntaxScanner {
  fn pre_module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    let Some(decl) = decl.as_import() else {
      return None;
    };
    collect_from_import_decl(&self.caps, decl, parser.worker_syntax_list);
    None
  }
}
