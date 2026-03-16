use either::Either;
use rspack_swc_plugin_import::ImportOptions;
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
pub(crate) fn transform<'a>(
  transform_import: &'a Option<Vec<ImportOptions>>,
  rspack_experiments: &'a RspackExperiments,
) -> impl Pass + 'a {
  // Prefer top-level `transformImport` over deprecated `rspackExperiments.import`
  let import_options = transform_import
    .as_ref()
    .or(rspack_experiments.import.as_ref());
  either!(import_options, |options| {
    rspack_swc_plugin_import::plugin_import(options)
  })
}
