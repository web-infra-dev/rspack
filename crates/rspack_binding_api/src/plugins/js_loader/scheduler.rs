use napi::bindgen_prelude::JsValuesTupleIntoVec;
use rspack_core::{
  BUILTIN_LOADER_PREFIX, LoaderContext, NormalModuleLoaderShouldYield,
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
