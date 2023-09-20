use std::path::{Path, PathBuf};
use std::sync::Arc;

use either::Either;
use rspack_core::CompilerOptions;
use rspack_swc_visitors::{styled_components, StyledComponentsOptions};
use swc_core::common::FileName;
use swc_core::common::{chain, comments::Comments, Mark, SourceMap};
use swc_core::ecma::{transforms::base::pass::noop, visit::Fold};

use crate::options::RspackExperiments;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
  ($config:expr, $f:expr, $enabled:expr) => {
    if $enabled {
      either!($config, $f)
    } else {
      Either::Right(noop())
    }
  };
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform<'a>(
  resource_path: &'a Path,
  rspack_options: &'a CompilerOptions,
  comments: Option<&'a dyn Comments>,
  _top_level_mark: Mark,
  unresolved_mark: Mark,
  cm: Arc<SourceMap>,
  content_hash: Option<u32>,
  rspack_experiments: &'a RspackExperiments,
) -> impl Fold + 'a {
  use rspack_swc_visitors::EmotionOptions;

  chain!(
    either!(rspack_experiments.emotion, |options: &EmotionOptions| {
      // SAFETY: Source content hash should always available if emotion is turned on.
      let content_hash = content_hash.expect("Content hash should be available");
      rspack_swc_visitors::emotion(
        options.clone(),
        resource_path,
        content_hash,
        cm.clone(),
        comments,
      )
    }),
    either!(
      rspack_experiments.styled_components,
      |options: &StyledComponentsOptions| {
        let content_hash = content_hash.expect("Content hash should be available");
        styled_components(
          FileName::Real(resource_path.into()),
          content_hash.into(),
          options.clone(),
        )
      }
    ),
    either!(rspack_experiments.relay, |options| {
      rspack_swc_visitors::relay(
        options,
        resource_path,
        PathBuf::from(AsRef::<Path>::as_ref(&rspack_options.context)),
        unresolved_mark,
      )
    }),
    either!(rspack_experiments.import, |options| {
      rspack_swc_visitors::import(options)
    }),
  )
}
