use either::Either;
use swc_core::ecma::ast::{Pass, noop_pass};

use crate::options::RspackExperiments;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
      Either::Left($f(config))
    } else {
      Either::Right(noop_pass())
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
pub(crate) fn transform(rspack_experiments: &RspackExperiments) -> impl Pass + '_ {
  (
    // Legacy import API (deprecated)
    either!(rspack_experiments.import, |options| {
      rspack_swc_plugin_import::plugin_import(options)
    }),
    // Modern transformImport API
    either!(rspack_experiments.transform_import, |options| {
      rspack_swc_plugin_import::transform_import(options)
    }),
  )
}
