use std::sync::Arc;

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
  mut loader_context: Arc<LoaderContext<RunnerContext>>,
) -> (Arc<LoaderContext<RunnerContext>>, Result<()>) {
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
      let context =
        Arc::get_mut(&mut loader_context).expect("native loader context must be exclusive");
      for index in start..end {
        context.loader_items[index].set_pitch_executed();
      }
      context.loader_index = end as i32;
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

  let pitching = loader_context.state() == LoaderState::Pitching;
  let content = Arc::get_mut(&mut loader_context)
    .expect("native loader context must be exclusive")
    .take_content();
  let result = async {
    runner
      .call_async(super::context::JsLoaderContextTransfer {
        context: Arc::downgrade(&loader_context),
        content,
      })
      .await?
      .await
  }
  .await;
  let updates = match result {
    Ok(updates) => updates,
    // Errors stop the run; consuming its Arc invalidates any retained JS Weak.
    Err(error) => return (loader_context, Err(error).to_rspack_result()),
  };
  if pitching {
    let loaders_without_pitch = updates.collect_loaders_without_pitch(&loader_context);
    if !loaders_without_pitch.is_empty() {
      self
        .update_loaders_without_pitch(loaders_without_pitch)
        .await;
    }
  }
  // TODO: Prefer Arc::into_unique here once Rust implements it.
  let result = updates.merge(
    Arc::get_mut(&mut loader_context)
      .expect("JavaScript must release the loader context before returning"),
  );
  (loader_context, result)
}
