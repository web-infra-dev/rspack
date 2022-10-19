use std::sync::Arc;
use swc_common::{Mark, SourceMap, SyntaxContext};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_lints::rules::{all, lint_to_fold, LintParams};
use swc_ecma_visit::Fold;

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
