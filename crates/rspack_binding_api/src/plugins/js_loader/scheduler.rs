use napi::JsValue;
use rspack_core::{LoaderContext, NormalModuleLoaderStartYielding, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State;
use tokio::sync::oneshot;

use super::{
  JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner, result::merge_loader_context,
  runtime::ExecutionMode,
};
use crate::js_tasks::{JsTask, LoaderTask, js_task_dispatcher};

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin, tracing=false)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  let runtime = &self.runtime;
  if loader_context.state() == State::Pitching {
    let without_pitch = runtime
      .loaders_without_pitch
      .lock()
      .expect("loader pitch cache lock poisoned");
    let end = loader_context
      .current_chain()
      .expect("pitching requires a current execution chain")
      .end();
    let start = loader_context.loader_index as usize;
    let needs_pitch = loader_context.loader_items[start..end]
      .iter()
      .any(|loader| !loader.pitch_executed() && !without_pitch.contains(loader.path().as_str()));
    if !needs_pitch {
      for loader in &loader_context.loader_items[start..end] {
        loader.set_pitch_executed();
      }
      loader_context.loader_index = end as i32;
      return Ok(());
    }
  }
  let (start, end, mode) = {
    let current = loader_context.loader_index as usize;
    let mode_at = |index: usize| runtime.execution_mode(loader_context.loader_items[index].query());
    let mode = mode_at(current);
    let chain = loader_context
      .current_chain()
      .expect("yield requires a loader chain");
    let mut start = current;
    let mut end = current + 1;
    while start > chain.start() && mode_at(start - 1) == mode {
      start -= 1;
    }
    while end < chain.end() && mode_at(end) == mode {
      end += 1;
    }
    (start, end, mode)
  };
  if mode == ExecutionMode::Strict {
    loader_context.take_additional_data();
  }
  let mut context = JsLoaderContext::try_from(&mut *loader_context)?;
  context.loader_chain_start = start as u32;
  context.loader_chain_end = end as u32;
  let context = if mode == ExecutionMode::Main {
    let runner = self.runner.lock().expect("should get lock").clone();
    let runner = runner
      .get_or_try_init(|| async {
        let compiler_id = self
          .compiler_id
          .get()
          .expect("compiler is initialized before loaders");
        self.runner_getter.call(compiler_id).await
      })
      .await
      .to_rspack_result()?;
    runner
      .call_with_promise_prepared(move |env| {
        let (context, access) = context.into_jsobject(env)?;
        Ok((vec![context.raw()], access))
      })
      .await?
  } else {
    if let Some(module) = context.module.as_mut() {
      module.worker = true;
    }
    let (result, reply) = oneshot::channel();
    js_task_dispatcher().dispatch_worker(JsTask::Loader(LoaderTask {
      context,
      compiler_handle: runtime.main_object_handle,
      resolver_factory: (mode == ExecutionMode::Strict)
        .then(|| loader_context.context.resolver_factory.clone()),
      result,
    }));
    reply
      .await
      .map_err(|_| error!("Parallel loader task was cancelled"))??
  };
  if mode != ExecutionMode::Main || loader_context.state() == State::Pitching {
    let mut without_pitch = context
      .loader_items
      .iter()
      .zip(loader_context.loader_items.iter())
      .filter(|(js, _)| js.no_pitch)
      .map(|(_, native)| native.path().as_str())
      .peekable();
    if without_pitch.peek().is_some() {
      runtime
        .loaders_without_pitch
        .lock()
        .expect("loader pitch cache lock poisoned")
        .extend(without_pitch.map(str::to_owned));
    }
  }
  merge_loader_context(loader_context, context)
}
