use napi::JsValue;
use rspack_core::{LoaderContext, NormalModuleLoaderStartYielding, RunnerContext};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{
  JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner, result::merge_loader_context,
};

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
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
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
      for loader in &mut loader_context.loader_items[start..end] {
        loader.set_pitch_executed();
      }
      loader_context.loader_index = end as i32;
      return Ok(());
    }
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

  let context = JsLoaderContext::try_from(&mut *loader_context)?;
  let new_cx = runner
    .call_with_promise_prepared(move |env| {
      let (context, access) = context.into_jsobject(env)?;
      Ok((vec![context.raw()], access))
    })
    .await?;

  if loader_context.state() == LoaderState::Pitching {
    let list = collect_loaders_without_pitch(loader_context, &new_cx);
    if !list.is_empty() {
      self.update_loaders_without_pitch(list).await;
    }
  }

  merge_loader_context(loader_context, new_cx)?;

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
