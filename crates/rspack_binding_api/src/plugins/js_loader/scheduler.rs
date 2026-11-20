use napi::JsValue;
use rspack_core::{
  JavaScriptExecutionMode, LoaderContext, LoaderExecutionKind, NormalModuleLoaderStartYielding,
  RunnerContext,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State;
use tokio::sync::oneshot;

use super::{
  JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner, result::merge_loader_context,
};
use crate::js_tasks::{JsTask, LoaderTask};

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin, tracing=false)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  let compiler_id = *self
    .compiler_id
    .get()
    .expect("compiler is initialized before loaders");
  let runtime = super::bridge::loader_runtime(compiler_id.as_u32());
  let chain = loader_context
    .current_chain()
    .expect("yield requires a current execution chain");
  let start = chain.start();
  let end = chain.end();
  let mode = chain
    .execution_kind()
    .and_then(LoaderExecutionKind::javascript_mode)
    .expect("yield requires a JavaScript execution chain");
  if loader_context.state() == State::Pitching {
    let without_pitch = runtime
      .loaders_without_pitch
      .lock()
      .expect("loader pitch cache lock poisoned");
    let current = loader_context.loader_index as usize;
    let needs_pitch = loader_context.loader_items[current..end]
      .iter()
      .any(|loader| !loader.pitch_executed() && !without_pitch.contains(loader.path().as_str()));
    if !needs_pitch {
      for loader in &loader_context.loader_items[current..end] {
        loader.set_pitch_executed();
      }
      loader_context.loader_index = end as i32;
      return Ok(());
    }
  }
  let needs_serialization = mode == JavaScriptExecutionMode::Main
    && loader_context.loader_items[..start].iter().any(|loader| {
      loader.execution_kind() == LoaderExecutionKind::JavaScript(JavaScriptExecutionMode::Parallel)
    });
  let mut context =
    JsLoaderContext::from_loader_context(&mut *loader_context, mode, needs_serialization)?;
  let context = if mode == JavaScriptExecutionMode::Main {
    let runner = self.runner.lock().expect("should get lock").clone();
    let runner = runner
      .get_or_try_init(|| async { self.runner_getter.call(&compiler_id).await })
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
    self.dispatcher.dispatch_worker(JsTask::Loader(LoaderTask {
      context,
      compiler_id: compiler_id.as_u32(),
      resolver_factory: (mode == JavaScriptExecutionMode::Strict)
        .then(|| loader_context.context.resolver_factory.clone()),
      result,
    }));
    reply
      .await
      .map_err(|_| error!("Parallel loader task was cancelled"))??
  };
  if mode != JavaScriptExecutionMode::Main || loader_context.state() == State::Pitching {
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
