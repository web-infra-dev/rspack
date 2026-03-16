use either::Either;
use rspack_swc_plugin_import::ImportOptions;
use swc_core::ecma::ast::{Pass, noop_pass};

use crate::options::RspackExperiments;

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform<'a>(
  transform_import: &'a Option<Vec<ImportOptions>>,
  rspack_experiments: &'a RspackExperiments,
) -> impl Pass + 'a {
  // Prefer top-level `transformImport` over deprecated `rspackExperiments.import`
  let (import_options, option_name) = if transform_import.is_some() {
    (transform_import.as_ref(), "transformImport")
  } else {
    (
      rspack_experiments.import.as_ref(),
      "rspackExperiments.import",
    )
  };
  if let Some(options) = import_options {
    Either::Left(rspack_swc_plugin_import::plugin_import(
      options,
      option_name,
    ))
  } else {
    Either::Right(noop_pass())
  }
}
