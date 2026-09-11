use napi::bindgen_prelude::JsValuesTupleIntoVec;
use rspack_core::{LoaderContext, NormalModuleLoaderStartYielding, RunnerContext};
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

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin,tracing=false)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut Option<Box<LoaderContext<RunnerContext>>>,
) -> Result<()> {
  let cx = loader_context
    .as_mut()
    .expect("loader context is available");
  // Keep pitch capability discovery on the JS side of the runtime boundary.
  // A loader known not to have a pitch function does not need a JS callback.
  if cx.state() == LoaderState::Pitching
    && self
      .loaders_without_pitch
      .read()
      .await
      .contains(cx.current_loader().path().as_str())
  {
    cx.set_current_loader_pitch_executed();
    cx.loader_index += 1;
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

  let mut js_context = runner
    .call_async(JsLoaderContext::new(
      loader_context.take().expect("loader context is available"),
    ))
    .await
    .to_rspack_result()?
    .await
    .to_rspack_result()?;
  *loader_context = js_context.context.take();

  if !js_context.loaders_without_pitch.is_empty() {
    self
      .update_loaders_without_pitch(std::mem::take(&mut js_context.loaders_without_pitch))
      .await;
  }
  js_context.take_error()
}
