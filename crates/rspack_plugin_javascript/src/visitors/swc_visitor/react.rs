use rspack_core::ReactOptions;
use std::sync::Arc;
use swc_common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_ecma_transforms::react::{react as swc_react, Options};
use swc_ecma_visit::Fold;

pub fn react<'a>(
  top_level_mark: Mark,
  comments: Option<&'a SingleThreadedComments>,
  cm: &Arc<SourceMap>,
  options: &ReactOptions,
) -> impl Fold + 'a {
  swc_react(
    cm.clone(),
    comments,
    Options {
      refresh: None,
      runtime: options.runtime,
      import_source: options.import_source.clone(),
      pragma: options.pragma.clone(),
      pragma_frag: options.pragma_frag.clone(),
      throw_if_namespace: options.throw_if_namespace,
      development: options.development,
      use_builtins: options.use_builtins,
      use_spread: options.use_spread,
      ..Default::default()
    },
    top_level_mark,
  )
}
