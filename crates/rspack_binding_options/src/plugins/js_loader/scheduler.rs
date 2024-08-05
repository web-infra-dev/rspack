use rspack_core::{
  LoaderContext, NormalModuleLoaderShouldYield, NormalModuleLoaderStartYielding, RunnerContext,
  BUILTIN_LOADER_PREFIX,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContextInstance, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[plugin_hook(NormalModuleLoaderShouldYield for JsLoaderRspackPlugin)]
pub(crate) fn loader_should_yield(
  &self,
  loader_context: &LoaderContext<RunnerContext>,
) -> Result<Option<bool>> {
  match loader_context.state() {
    s @ LoaderState::Init | s @ LoaderState::ProcessResource | s @ LoaderState::Finished => {
      panic!("Unexpected loader runner state: {s:?}")
    }
    LoaderState::Pitching | LoaderState::Normal => {
      return Ok(Some(
        !loader_context
          .current_loader()
          .request()
          .starts_with(BUILTIN_LOADER_PREFIX),
      ))
    }
  }
}

#[plugin_hook(NormalModuleLoaderStartYielding for JsLoaderRspackPlugin)]
pub(crate) async fn loader_yield(
  &self,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  let js_loader_context = JsLoaderContextInstance::from(loader_context);
  let _ = self.runner.call_with_promise(js_loader_context).await?;
  Ok(())
}
