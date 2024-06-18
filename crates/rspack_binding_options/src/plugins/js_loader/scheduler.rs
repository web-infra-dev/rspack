use napi::Either;
use rspack_core::{
  LoaderContext, NormalModuleLoaderShouldYield, NormalModuleLoaderStartYielding, RunnerContext,
  BUILTIN_LOADER_PREFIX,
};
use rspack_error::{error, Result};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[plugin_hook(NormalModuleLoaderShouldYield for JsLoaderRspackPlugin)]
pub(crate) fn loader_should_yield(
  &self,
  loader_context: &LoaderContext<RunnerContext>,
) -> Result<Option<bool>> {
  match loader_context.state() {
    s @ LoaderState::Init | s @ LoaderState::ProcessResource | s @ LoaderState::Finished => {
      panic!("Unexpected loader runner state: {s:?}")
    }
    LoaderState::Pitching | LoaderState::Normal => {
      return Ok(Some(
        !loader_context
          .current_loader()
          .request()
          .starts_with(BUILTIN_LOADER_PREFIX),
      ))
    }
  }
}

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  let new_cx = self
    .runner
    .call_with_promise(loader_context.try_into()?)
    .await?;
  merge_loader_context(loader_context, new_cx)?;
  Ok(())
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  if let Some(data) = &from.additional_data {
    to.additional_data.insert(data.clone());
  }
  to.cacheable = from.cacheable;
  to.file_dependencies = from
    .file_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.context_dependencies = from
    .context_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.missing_dependencies = from
    .missing_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.build_dependencies = from
    .build_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.content = match from.content {
    Either::A(_) => None,
    Either::B(c) => Some(rspack_core::Content::from(Into::<Vec<u8>>::into(c))),
  };
  to.source_map = from
    .source_map
    .as_ref()
    .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
    .transpose()
    .map_err(|e| error!(e.to_string()))?;
  to.asset_filenames = from.asset_filenames.into_iter().collect();

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
      to.set_data(from.data);
      to
    })
    .collect();
  to.loader_index = from.loader_index;

  Ok(())
}
