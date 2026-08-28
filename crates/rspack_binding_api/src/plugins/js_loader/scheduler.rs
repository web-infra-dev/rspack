use napi::{Either, bindgen_prelude::JsValuesTupleIntoVec};
use rspack_core::{
  AdditionalData, BUILTIN_LOADER_PREFIX, LoaderContext, NormalModuleLoaderShouldYield,
  NormalModuleLoaderStartYielding, RunnerContext,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::MainThreadJsValueHandle;
use rspack_tasks::WorkerFailure;
use tracing::{Instrument, info_span};

use super::{JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};
use crate::worker::{WorkerTaskPayload, dispatch_worker_task};

impl JsLoaderRspackPlugin {
  async fn update_loaders_without_pitch(&self, list: Vec<String>) {
    let mut loaders_without_pitch = self.loaders_without_pitch.write().await;
    for path in list {
      loaders_without_pitch.insert(path);
    }
  }

  async fn run_on_main(
    &self,
    context: &mut LoaderContext<RunnerContext>,
    hooks_only: bool,
  ) -> Result<JsLoaderContext> {
    let runner = self.runner.lock().expect("should get lock").clone();
    let runner = runner
      .get_or_try_init(|| async {
        #[allow(clippy::unwrap_used)]
        let compiler_id = self.compiler_id.get().unwrap();
        self.runner_getter.call(compiler_id).await
      })
      .await
      .to_rspack_result()?;
    let mut js_context: JsLoaderContext = context.try_into()?;
    js_context.run_hooks_only = hooks_only;
    runner
      .call_async(js_context)
      .await
      .to_rspack_result()?
      .await
      .to_rspack_result()
  }
}

#[plugin_hook(NormalModuleLoaderShouldYield for JsLoaderRspackPlugin, tracing=false)]
pub(crate) async fn loader_should_yield(
  &self,
  loader_context: &LoaderContext<RunnerContext>,
) -> Result<Option<bool>> {
  match loader_context.state() {
    s @ (LoaderState::Init | LoaderState::ProcessResource | LoaderState::Finished) => {
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
  loader_context: &mut Option<Box<LoaderContext<RunnerContext>>>,
) -> Result<()> {
  let mut context = loader_context
    .take()
    .expect("loader_yield hook owns LoaderContext while it is executing");
  let is_pitching = context.state() == LoaderState::Pitching;
  let parallel = context.current_loader().parallel()
    && !(std::env::var_os("WASM").is_some() && context.current_loader().cache());

  let result = if parallel {
    let hook_result = self
      .run_on_main(context.as_mut(), true)
      .instrument(info_span!("JsLoader:main_hooks"))
      .await;
    match hook_result {
      Err(error) => Err(error),
      Ok(mut hook_context) => {
        let hook_extensions = hook_context.hook_extensions.take();
        merge_loader_hook_context(context.as_mut(), hook_context);
        let result = dispatch_worker_task(Box::new(WorkerTaskPayload {
          loader_context: *context,
          loaders_without_pitch: Vec::new(),
          hook_extensions,
        }))
        .instrument(info_span!("JsLoader:queue_wait_and_execute"))
        .await;
        let (mut payload, error) = match result {
          Ok(payload) => (payload, None),
          Err(failure) => {
            let (error, payload) = failure.into_parts();
            (
              payload.expect("worker must return its Rust LoaderContext"),
              Some(match error {
                WorkerFailure::Dispatch(error) => rspack_error::error!(error.to_string()),
                WorkerFailure::Task(error) => error,
              }),
            )
          }
        };
        context = Box::new(payload.loader_context);
        if is_pitching && !payload.loaders_without_pitch.is_empty() {
          self
            .update_loaders_without_pitch(std::mem::take(&mut payload.loaders_without_pitch))
            .await;
        }
        error.map_or(Ok(()), Err)
      }
    }
  } else {
    let result = async {
      let new_context = self.run_on_main(context.as_mut(), false).await?;
      if is_pitching {
        let list = collect_loaders_without_pitch(&context, &new_context);
        if !list.is_empty() {
          self.update_loaders_without_pitch(list).await;
        }
      }
      merge_loader_context(&mut context, new_context)
    }
    .instrument(info_span!("JsLoader:main_execute"))
    .await;
    result
  };

  *loader_context = Some(context);
  result
}

fn merge_loader_hook_context(to: &mut LoaderContext<RunnerContext>, mut from: JsLoaderContext) {
  to.cacheable = from.cacheable;
  to.replace_dependencies(from.dependencies.into());
  for (to, from) in to.loader_items.iter_mut().zip(from.loader_items.drain(..)) {
    to.set_data(from.data);
  }
  to.parse_meta.extend(
    from
      .parse_meta
      .into_iter()
      .map(|(key, value)| (key, Box::new(value) as _)),
  );
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  to.cacheable = from.cacheable;
  to.replace_dependencies(from.dependencies.into());

  if let Some(error) = from.error {
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
  // Wrap the JS registry handle before any fallible conversion below so an early return also
  // schedules registry cleanup.
  let additional_data = from.additional_data.take().map(|handle| {
    let mut additional = AdditionalData::default();
    additional.insert::<MainThreadJsValueHandle>(MainThreadJsValueHandle::new(handle));
    additional
  });
  let source_map = from
    .source_map
    .map(|buffer| rspack_core::rspack_sources::SourceMap::from_bytes(buffer.into()))
    .transpose()
    .to_rspack_result()?;
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
