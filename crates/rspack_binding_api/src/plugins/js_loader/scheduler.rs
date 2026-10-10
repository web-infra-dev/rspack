use rspack_core::{NormalModuleLoaderStartYielding, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::{LoaderContext, State as LoaderState};

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

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
  mut loader_context: Box<LoaderContext<RunnerContext>>,
) -> (Box<LoaderContext<RunnerContext>>, Result<()>) {
  // Skip a JavaScript execution span when no remaining loader needs pitching.
  if loader_context.state() == LoaderState::Pitching {
    let loaders_without_pitch = self.loaders_without_pitch.read().await;
    let end = loader_context
      .current_chain()
      .expect("pitching requires a current execution chain")
      .end();
    let start = loader_context.loader_index as usize;
    let needs_pitch = loader_context.loader_items[start..end]
      .iter()
      .enumerate()
      .any(|(offset, loader)| {
        !loader_context.loader_items[start + offset].pitch_executed()
          && !loaders_without_pitch.contains(loader.path().as_str())
      });
    if !needs_pitch {
      for index in start..end {
        loader_context.loader_items[index].set_pitch_executed();
      }
      loader_context.loader_index = end as i32;
      return (loader_context, Ok(()));
    }
  }

  let runner = self.runner.lock().expect("should get lock").clone();
  let runner = match runner
    .get_or_try_init(|| async {
      #[allow(clippy::unwrap_used)]
      let compiler_id = self.compiler_id.get().unwrap();
      self.runner_getter.call(compiler_id).await
    })
    .await
    .to_rspack_result()
  {
    Ok(runner) => runner,
    Err(error) => return (loader_context, Err(error)),
  };

  let id = loader_context.id();
  let pitching = loader_context.state() == LoaderState::Pitching;
  let pending = std::sync::Arc::new(std::sync::Mutex::new(Some(loader_context)));
  let transfer_guard =
    super::context::LoaderContextTransferGuard::new(id, std::sync::Arc::clone(&pending));
  let result = async {
    runner
      .call_async(super::context::JsLoaderContextTransfer(
        std::sync::Arc::clone(&pending),
      ))
      .await?
      .await
  }
  .await;
  let (new_cx, runner_error) = match result {
    Ok(context) => (context, None),
    Err(error) => {
      let context = pending.lock().expect("pending loader context lock").take();
      if let Some(context) = context {
        transfer_guard.disarm();
        return (context, Err(error).to_rspack_result());
      }
      (
        self
          .runner_getter
          .reclaim(id)
          .await
          .expect("a rejected loader runner must return its owned context"),
        Some(error),
      )
    }
  };
  loader_context = new_cx.context;
  transfer_guard.disarm();
  if let Some(error) = runner_error {
    return (loader_context, Err(error).to_rspack_result());
  }
  let updates = match new_cx.updates.to_rspack_result() {
    Ok(updates) => updates,
    Err(error) => return (loader_context, Err(error)),
  };
  if pitching {
    let loaders_without_pitch = updates.collect_loaders_without_pitch(&loader_context);
    if !loaders_without_pitch.is_empty() {
      self
        .update_loaders_without_pitch(loaders_without_pitch)
        .await;
    }
  }
  let result = updates.merge(&mut loader_context);
  (loader_context, result)
}
