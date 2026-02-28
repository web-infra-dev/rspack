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
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform(rspack_experiments: &RspackExperiments) -> impl Pass + '_ {
  either!(rspack_experiments.import, |options| {
    rspack_swc_plugin_import::plugin_import(options)
  })
}
