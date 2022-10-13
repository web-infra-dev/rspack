use std::sync::Arc;
use swc_common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_ecma_transforms::react::{react as swc_react, Options, Runtime};
use swc_ecma_visit::Fold;

pub fn react<'a>(
  top_level_mark: Mark,
  comments: Option<&'a SingleThreadedComments>,
  cm: &Arc<SourceMap>,
) -> impl Fold + 'a {
  swc_react(
    cm.clone(),
    comments,
    Options {
      development: Some(false),
      runtime: Some(Runtime::Classic),
      refresh: None,
      ..Default::default()
    },
    top_level_mark,
  )
}
