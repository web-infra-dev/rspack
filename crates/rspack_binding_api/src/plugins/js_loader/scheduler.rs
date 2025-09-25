use napi::Either;
use rspack_core::{
  AdditionalData, BUILTIN_LOADER_PREFIX, LoaderContext, NormalModuleLoaderShouldYield,
  NormalModuleLoaderStartYielding, RunnerContext,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

impl JsLoaderRspackPlugin {
  async fn update_loaders_without_pitch(&self, list: Vec<String>) {
    let mut loaders_without_pitch = self.loaders_without_pitch.write().await;
    for path in list {
      loaders_without_pitch.insert(path);
    }
  }
}

#[plugin_hook(NormalModuleLoaderShouldYield for JsLoaderRspackPlugin, tracing=false)]
pub(crate) async fn loader_should_yield(
  &self,
  loader_context: &LoaderContext<RunnerContext>,
) -> Result<Option<bool>> {
  match loader_context.state() {
    s @ LoaderState::Init | s @ LoaderState::ProcessResource | s @ LoaderState::Finished => {
      panic!("Unexpected loader runner state: {s:?}")
    }
    LoaderState::Pitching => {
      let current_loader = loader_context.current_loader();
      if current_loader.request().starts_with(BUILTIN_LOADER_PREFIX) {
        Ok(Some(false))
      } else {
        let loaders_without_pitch = self.loaders_without_pitch.read().await;
        let should_yield = !loaders_without_pitch.contains(current_loader.path().as_str());
        Ok(Some(should_yield))
      }
    }
    LoaderState::Normal => Ok(Some(
      !loader_context
        .current_loader()
        .request()
        .starts_with(BUILTIN_LOADER_PREFIX),
    )),
  }
}

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin,tracing=false)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
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

  merge_loader_context(loader_context, new_cx)?;

  Ok(())
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  to.cacheable = from.cacheable;
  to.file_dependencies = from.file_dependencies.into_iter().map(Into::into).collect();
  to.context_dependencies = from
    .context_dependencies
    .into_iter()
    .map(Into::into)
    .collect();
  to.missing_dependencies = from
    .missing_dependencies
    .into_iter()
    .map(Into::into)
    .collect();
  to.build_dependencies = from
    .build_dependencies
    .into_iter()
    .map(Into::into)
    .collect();

  if let Some(error) = from.error {
    let details = if let Some(stack) = &error.stack
      && error.hide_stack.unwrap_or(false)
    {
      Some(stack.to_string())
    } else {
      None
    };
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
    .as_ref()
    .map(|s| {
      rspack_core::rspack_sources::SourceMap::from_json(
        // SAFETY: `sourceMap` is serialized by JavaScript from a JSON object. This is an invariant should be followed on the JavaScript side.
        unsafe { str::from_utf8_unchecked(s) },
      )
    })
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
