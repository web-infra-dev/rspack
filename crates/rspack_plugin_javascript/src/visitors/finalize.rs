// use super::hmr::HmrModuleIdReWriter;
use crate::visitors::RspackModuleFinalizer;
use rspack_core::{Compilation, ModuleGraphModule};
use swc_common::{chain, Mark, DUMMY_SP};
use swc_ecma_transforms::resolver;
use {swc_common, swc_ecma_utils::quote_ident, swc_ecma_visit::Fold};

pub fn finalize<'a>(
  module: &'a ModuleGraphModule,
  compilation: &'a Compilation,
  // entry_flag: bool,
) -> impl Fold + 'a {
  let (unresolved_mark, top_level_mark) = (Mark::new(), Mark::new());
  let finalize_pass = chain!(
    // We assume the AST is cleaned by default
    // as_folder(ClearMark),
    resolver(unresolved_mark, top_level_mark, false,),
    RspackModuleFinalizer {
      module,
      require_ident: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "__rspack_require__"),
      module_ident: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "module"),
      // entry_flag,
      compilation,
    },
    // as_folder(HmrModuleIdReWriter {
    //   resolved_ids,
    //   rewriting: false,
    //   bundle,
    // })
  );
  finalize_pass
}
