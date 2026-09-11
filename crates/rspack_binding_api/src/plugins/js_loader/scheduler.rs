use napi::bindgen_prelude::JsValuesTupleIntoVec;
use rspack_core::{LoaderContext, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContext, JsLoaderRspackPlugin};

impl JsLoaderRspackPlugin {
  async fn update_loaders_without_pitch(&self, list: Vec<String>) {
    let mut loaders_without_pitch = self.loaders_without_pitch.write().await;
    for path in list {
      loaders_without_pitch.insert(path);
    }
  }
}

#[async_trait::async_trait]
impl rspack_loader_runner::LoaderRunner for JsLoaderRspackPlugin {
  type Context = RunnerContext;

  async fn run(
    &self,
    mut cx: Box<LoaderContext<RunnerContext>>,
  ) -> (Box<LoaderContext<RunnerContext>>, Result<()>) {
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
      return (cx, Ok(()));
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
      Err(error) => return (cx, Err(error)),
    };

    // Once transferred, the JS boundary must return the class even when a loader
    // throws. Transport failures cannot recover an allocation already sent to JS.
    let mut js_context = runner
      .call_async(JsLoaderContext::new(cx))
      .await
      .expect("JavaScript loader call must return the owned context")
      .await
      .expect("JavaScript loader promise must return the owned context");
    let cx = js_context
      .context
      .take()
      .expect("JavaScript returned the loader context");
    if !js_context.loaders_without_pitch.is_empty() {
      self
        .update_loaders_without_pitch(std::mem::take(&mut js_context.loaders_without_pitch))
        .await;
    }
    (cx, js_context.take_error())
  }
}
