use napi::bindgen_prelude::{Either3, JsValuesTupleIntoVec};
use rspack_core::{AdditionalData, LoaderContext, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContext, LoaderDispatcher};

pub(crate) async fn run_loaders(loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
  let dispatcher = LoaderDispatcher::get(loader_context.context.compiler_id)?;
  // Keep pitch capability discovery on the JS side of the runtime boundary.
  // A loader known not to have a pitch function does not need a JS callback.
  if loader_context.state() == LoaderState::Pitching
    && dispatcher
      .loaders_without_pitch
      .read()
      .expect("should get lock")
      .contains(loader_context.current_loader().path().as_str())
  {
    loader_context.current_loader().set_pitch_executed();
    loader_context.loader_index += 1;
    return Ok(());
  }

  let new_cx = dispatcher.run(loader_context.try_into()?).await?;

  if loader_context.state() == LoaderState::Pitching {
    let list = collect_loaders_without_pitch(loader_context, &new_cx);
    if !list.is_empty() {
      dispatcher
        .loaders_without_pitch
        .write()
        .expect("should get lock")
        .extend(list);
    }
  }

  merge_loader_context(loader_context, new_cx)?;

  Ok(())
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  to.cacheable = from.cacheable;
  to.replace_dependencies(from.dependencies.into());

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

fn collect_loaders_without_pitch(
  ctx: &LoaderContext<RunnerContext>,
  js_ctx: &JsLoaderContext,
) -> Vec<String> {
  let mut list = Vec::new();
  for (js_loader_item, loader_item) in js_ctx.loader_items.iter().zip(ctx.loader_items.iter()) {
    if js_loader_item.no_pitch {
      list.push(loader_item.path().to_string());
    }
  }
  list
}
