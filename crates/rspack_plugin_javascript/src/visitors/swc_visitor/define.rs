use std::sync::Arc;

use rspack_core::Define;
use swc_core::common::collections::AHashMap;
use swc_core::ecma::parser::EsConfig;
use swc_core::ecma::transforms::optimization::inline_globals2;
use swc_core::ecma::utils::NodeIgnoringSpan;
use swc_core::ecma::visit::Fold;
use swc_core::{
  common::FileName,
  ecma::parser::{parse_file_as_expr, Syntax},
};

pub fn define(opts: &Define) -> impl Fold {
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

      (NodeIgnoringSpan::owned(*target), *replacement)
    })
    .collect::<AHashMap<_, _>>();

  inline_globals2(
    Default::default(),
    Default::default(),
    Arc::new(defs),
    Default::default(),
  )
}
