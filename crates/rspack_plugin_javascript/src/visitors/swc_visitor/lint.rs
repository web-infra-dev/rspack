use std::sync::Arc;
use swc_core::common::{Mark, SourceMap, SyntaxContext};
use swc_core::ecma::ast::{EsVersion, Program};
use swc_core::ecma::Fold;
use swc_ecma_lints::rules::{all, lint_to_fold, LintParams};

pub fn lint(
  ast: &Program,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  es_version: EsVersion,
  cm: &Arc<SourceMap>,
) -> impl Fold {
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  let top_level_ctxt = SyntaxContext::empty().apply_mark(top_level_mark);
  lint_to_fold(all(LintParams {
    program: ast,
    lint_config: &Default::default(),
    top_level_ctxt,
    unresolved_ctxt,
    es_version,
    source_map: cm.clone(),
  }))
}
