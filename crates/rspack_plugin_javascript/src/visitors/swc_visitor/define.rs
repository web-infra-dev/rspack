use std::sync::Arc;

use rspack_core::Define;
use swc_core::ecma::parser::EsConfig;
use swc_core::ecma::visit::Fold;
use swc_core::{
  common::FileName,
  ecma::parser::{parse_file_as_expr, Syntax},
};
use swc_core::{common::Mark, ecma::visit::as_folder};
use swc_ecma_minifier::globals_defs;

pub fn define(opts: &Define, unresolved_mark: Mark, top_level_mark: Mark) -> impl Fold {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let defs = opts
    .iter()
    .map(|(target, replacement)| {
      let target = {
        let fm = cm.new_source_file(FileName::Anon, target.clone());
        parse_file_as_expr(
          &fm,
          Syntax::Es(EsConfig::default()),
          rspack_core::EsVersion::EsNext,
          None,
          &mut vec![],
        )
        .unwrap_or_else(|_| panic!("builtins.define: Failed to parse {:?}", target))
      };
      let replacement = {
        let fm = cm.new_source_file(FileName::Anon, replacement.clone());
        parse_file_as_expr(
          &fm,
          Syntax::Es(EsConfig::default()),
          rspack_core::EsVersion::EsNext,
          None,
          &mut vec![],
        )
        .unwrap_or_else(|_| panic!("builtins.define: Failed to parse {:?}", target))
      };

      (target, replacement)
    })
    .collect::<Vec<_>>();

  as_folder(globals_defs(defs, unresolved_mark, top_level_mark))
}
