use std::path::{Path, PathBuf};
use std::sync::Arc;

use either::Either;
use rspack_core::CompilerOptions;
use swc_core::common::{chain, comments::Comments, Mark, SourceMap};
use swc_core::ecma::{
  transforms::base::pass::{noop, Optional},
  visit::Fold,
};
use swc_emotion::EmotionOptions;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
}

/// This should only be running at `custom_after_pass`, or
/// it will contain `resolve` issues.
pub fn transform<'a>(
  resource_path: &'a Path,
  options: &'a CompilerOptions,
  comments: Option<&'a dyn Comments>,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  cm: Arc<SourceMap>,
  content_hash: Option<u32>,
) -> impl Fold + 'a {
  let should_transform_by_react = true;

  // # Guarantee
  //
  // `swc` invokes `custom_before_pass` after
  //
  //  - Handling decorators, if configured
  //  - Applying `resolver`
  //  - Stripping typescript nodes
  chain!(
    Optional::new(
      rspack_swc_visitors::react(
        top_level_mark,
        comments,
        &cm,
        &options.builtins.react,
        unresolved_mark
      ),
      should_transform_by_react
    ),
    Optional::new(
      rspack_swc_visitors::fold_react_refresh(unresolved_mark),
      should_transform_by_react
        && options
          .builtins
          .react
          .refresh
          .and_then(|v| if v { Some(v) } else { None })
          .is_some()
    ),
    either!(
      options.builtins.emotion,
      |emotion_options: &EmotionOptions| {
        // SAFETY: Source content hash should always available if emotion is turned on.
        let content_hash = content_hash.unwrap();
        swc_emotion::emotion(
          emotion_options.clone(),
          resource_path,
          content_hash,
          cm.clone(),
          comments,
        )
      }
    ),
    either!(options.builtins.relay, |relay_option| {
      rspack_swc_visitors::relay(
        relay_option,
        resource_path,
        PathBuf::from(AsRef::<Path>::as_ref(&options.context)),
        unresolved_mark,
      )
    }),
    either!(options.builtins.plugin_import, |config| {
      swc_plugin_import::plugin_import(config)
    }),
    Optional::new(
      rspack_swc_visitors::define(&options.builtins.define),
      !options.builtins.define.is_empty()
    ),
    Optional::new(
      rspack_swc_visitors::provide_builtin(&options.builtins.provide, unresolved_mark),
      !options.builtins.provide.is_empty()
    ),
  )
}
