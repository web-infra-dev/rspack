// use super::hmr::HmrModuleIdReWriter;
use crate::visitors::RspackModuleFinalizer;
use rspack_core::{Compilation, Module};
use swc_common::{Mark, DUMMY_SP, GLOBALS};
use swc_ecma_utils::quote_ident;
use swc_ecma_visit::Fold;

pub fn finalize<'a>(
  module: &'a dyn Module,
  compilation: &'a Compilation,
  unresolved_mark: Mark,
) -> impl Fold + 'a {
  debug_assert!(GLOBALS.is_set());
  // let finalize_pass = chain!(
  // We assume the AST is cleaned by default
  // as_folder(ClearMark),
  // resolver(unresolved_mark, top_level_mark, false,),
  RspackModuleFinalizer {
    module,
    unresolved_mark,
    require_ident: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "__rspack_require__"),
    module_ident: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "module"),
    // entry_flag,
    compilation,
  }
  // as_folder(HmrModuleIdReWriter {
  //   resolved_ids,
  //   rewriting: false,
  //   bundle,
  // })
  // );
  // finalize_pass
}
