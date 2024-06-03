use std::path::{Path, PathBuf};
use std::sync::Arc;

use either::Either;
use once_cell::sync::Lazy;
use rspack_core::CompilerOptions;
use rspack_swc_visitors::{styled_components, StyledComponentsOptions};
use swc_core::atoms::Atom;
use swc_core::common::collections::AHashMap;
use swc_core::common::{
  chain,
  comments::{Comments, NoopComments},
  Mark, SourceMap,
};
use swc_core::common::{BytePos, FileName};
use swc_core::ecma::ast::Ident;
use swc_core::ecma::visit::{noop_visit_type, Visit};
use swc_core::ecma::{transforms::base::pass::noop, visit::Fold};
use xxhash_rust::xxh32::xxh32;

use crate::options::RspackExperiments;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
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
  content: &'a String,
  rspack_experiments: &'a RspackExperiments,
) -> impl Fold + 'a {
  use rspack_swc_visitors::EmotionOptions;
  let content_hash = Lazy::new(|| xxh32(content.as_bytes(), 0));

  chain!(
    either!(rspack_experiments.emotion, |options: &EmotionOptions| {
      rspack_swc_visitors::emotion(
        options.clone(),
        resource_path,
        *content_hash,
        cm.clone(),
        comments,
      )
    }),
    either!(
      rspack_experiments.styled_components,
      |options: &StyledComponentsOptions| {
        styled_components(
          FileName::Real(resource_path.into()),
          (*content_hash).into(),
          options.clone(),
          NoopComments,
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

pub struct IdentCollector {
  pub names: AHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
