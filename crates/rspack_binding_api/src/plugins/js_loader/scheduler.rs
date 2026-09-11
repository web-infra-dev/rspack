use napi::{Either, bindgen_prelude::JsValuesTupleIntoVec};
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

  let new_cx = runner
    .call_async(loader_context.try_into()?)
    .await
    .to_rspack_result()?
    .await
    .to_rspack_result()?;

  if loader_context.state() == LoaderState::Pitching {
    let list = collect_loaders_without_pitch(loader_context, &new_cx);
    if !list.is_empty() {
      self.update_loaders_without_pitch(list).await;
    }
  }

  merge_loader_state(loader_context, new_cx)?;

  Ok(())
}

pub(crate) fn merge_loader_state(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContextState,
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
    Either::A(_) => None,
    Either::B(c) => {
      // perf: Ignore UTF-8 check when JavaScript passed in an UTF-8 encoded value
      let content = if let Some(utf8_hint) = from.utf8_hint
        && utf8_hint
      {
        rspack_core::Content::from(
          // SAFETY: UTF-8 passed from JavaScript loader runner should ensure it does not pass non-UTF-8 encoded sequence when `utf_hint` is set to `true`. This invariant should be followed on the JavaScript side.
          unsafe { String::from_utf8_unchecked(c.into()) },
        )
      } else {
        rspack_core::Content::from(Into::<Vec<u8>>::into(c))
      };

      Some(content)
    }
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

  // update per-run loader status without mutating the shared loader metadata
  for (to, from) in to
    .loader_item_states
    .iter_mut()
    .zip(from.loader_item_states.drain(..))
  {
    if from.normal_executed {
      to.set_normal_executed();
      to.set_finish_called();
    }
    if from.pitch_executed {
      to.set_pitch_executed();
    }
    to.set_data(from.data);
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
