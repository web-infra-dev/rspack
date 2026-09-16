use napi::bindgen_prelude::{Either3, JsValuesTupleIntoVec};
use rspack_core::{AdditionalData, LoaderContext, NormalModuleLoaderStartYielding, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContextState, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

impl JsLoaderRspackPlugin {
  async fn update_loaders_without_pitch(&self, list: Vec<String>) {
    let mut loaders_without_pitch = self.loaders_without_pitch.write().await;
    for path in list {
      loaders_without_pitch.insert(path);
    }
  }
}

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin,tracing=false)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  // Keep pitch capability discovery on the JS side of the runtime boundary.
  // A loader known not to have a pitch function does not need a JS callback.
  if loader_context.state() == LoaderState::Pitching
    && self
      .loaders_without_pitch
      .read()
      .await
      .contains(loader_context.current_loader().path().as_str())
  {
    loader_context.set_current_loader_pitch_executed();
    loader_context.loader_index += 1;
    return Ok(());
  }

  let runner = self.runner.lock().expect("should get lock").clone();
  let runner = runner
    .get_or_try_init(|| async {
      #[allow(clippy::unwrap_used)]
      let compiler_id = self.compiler_id.get().unwrap();
      self.runner_getter.call(compiler_id).await
    })
    .await
    .to_rspack_result()?;

  let result = runner
    .call_async(loader_context.try_into()?)
    .await
    .to_rspack_result()?
    .await
    .to_rspack_result()?;

  if loader_context.state() == LoaderState::Pitching {
    let list = collect_loaders_without_pitch(loader_context, &result);
    if !list.is_empty() {
      self.update_loaders_without_pitch(list).await;
    }
  }

  merge_loader_state(loader_context, result)?;

  Ok(())
}

pub(crate) fn merge_loader_state(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContextState,
) -> Result<()> {
  if let Some(state) = from.loader_context_state.take() {
    to.context.loader_context_data.insert(state);
  }
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

  // Write back each loader's data and flags without touching its metadata.
  for ((to, data), from) in to
    .loader_item_states
    .iter_mut()
    .zip(&mut to.loader_data)
    .zip(from.loader_item_states.drain(..))
  {
    *data = from.data;
    if from.normal_executed {
      to.set_normal_executed();
      to.set_finish_called();
    }
    if from.pitch_executed {
      to.set_pitch_executed();
    }
  }
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
  js_ctx: &JsLoaderContextState,
) -> Vec<String> {
  let mut list = Vec::new();
  for (js_loader_item, loader_item) in js_ctx
    .loader_item_states
    .iter()
    .zip(ctx.loader_items().iter())
  {
    if js_loader_item.no_pitch {
      list.push(loader_item.path().to_string());
    }
  }
  list
}
