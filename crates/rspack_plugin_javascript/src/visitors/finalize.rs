// use super::hmr::HmrModuleIdReWriter;
use crate::visitors::RspackModuleFinalizer;
use rspack_core::{Compilation, Module};
use swc_core::common::{Mark, GLOBALS};
use swc_core::ecma::visit::Fold;

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
