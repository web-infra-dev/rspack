use std::collections::HashMap;

use super::hmr::HmrModuleIdReWriter;
use crate::{
  visitors::{ClearMark, RspackModuleFinalizer},
  Bundle, ModuleGraph, ResolvedURI,
};
use rspack_swc::{
  swc_atoms, swc_common,
  swc_ecma_transforms_base::resolver,
  swc_ecma_utils::quote_ident,
  swc_ecma_visit::{as_folder, Fold},
};
use swc_atoms::JsWord;
use swc_common::{chain, DUMMY_SP};

pub fn finalize<'a>(
  file_name: String,
  resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  entry_flag: bool,
  modules: &'a ModuleGraph,
  bundle: &'a Bundle,
) -> impl Fold + 'a {
  let finalize_pass = chain!(
    as_folder(ClearMark),
    resolver(
      bundle.context.unresolved_mark,
      bundle.context.top_level_mark,
      false,
    ),
    RspackModuleFinalizer {
      file_name,
      resolved_ids,
      require_ident: quote_ident!(
        DUMMY_SP.apply_mark(bundle.context.unresolved_mark),
        "__rspack_require__"
      ),
      module_ident: quote_ident!(
        DUMMY_SP.apply_mark(bundle.context.unresolved_mark),
        "module"
      ),
      entry_flag,
      modules,
      bundle,
    },
    as_folder(HmrModuleIdReWriter {
      resolved_ids,
      rewriting: false,
      bundle,
    })
  );
  finalize_pass
}
