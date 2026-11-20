use napi::bindgen_prelude::Either3;
use rspack_core::{AdditionalData, LoaderContext, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use super::JsLoaderContext;

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  to.cacheable = from.cacheable;
  to.replace_dependencies(
    from.dependencies.into(),
    from.added_dependencies.into(),
    from.removed_dependencies.into(),
  );
  for diagnostic in from.diagnostics.into_iter().flatten() {
    to.emit_diagnostic(diagnostic.into());
  }

  if let Some(error) = from.error {
    if let Some(diagnostic) = error.rust_diagnostic.as_ref() {
      return Err(diagnostic.error.clone());
    }
    return Err(error.with_parent_error_name("ModuleBuildError").into());
  }

  let content = match from.content {
    Either3::A(content) => Some(rspack_core::Content::String(content)),
    Either3::B(content) => Some(rspack_core::Content::Buffer(content.into())),
    Either3::C(_) => None,
  };
  let source_map = from
    .source_map
    .map(|buffer| rspack_core::rspack_sources::SourceMap::from_bytes(buffer.into()))
    .transpose()
    .to_rspack_result()?;
  let additional_data = from.additional_data.take().map(|data| {
    let mut additional = AdditionalData::default();
    additional.insert(data);
    additional
  });
  to.__finish_with((content, source_map, additional_data));

  // update loader status
  to.loader_items = to
    .loader_items
    .drain(..)
    .zip(from.loader_items.drain(..))
    .map(|(mut to, from)| {
      if from.normal_executed {
        to.set_normal_executed()
      }
      if from.pitch_executed {
        to.set_pitch_executed()
      }
      if from.filter_matched {
        to.set_filter_matched();
      }
      to.set_data(from.data);
      // JS loader should always be considered as finished
      to.set_finish_called();
      to
    })
    .collect();
  to.loader_index = from.loader_index;
  to.parse_meta.extend(
    from
      .parse_meta
      .into_iter()
      .map(|(k, v)| (k, Box::new(v) as _)),
  );

  Ok(())
}
