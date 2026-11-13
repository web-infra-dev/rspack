use std::sync::Arc;

use napi::bindgen_prelude::{Either3, JsValuesTupleIntoVec};
use rspack_core::{
  AdditionalData, LoaderContext, NormalModuleLoaderStartYielding, RunnerContext, StrictLoaderTask,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::{
  LoaderExecutionResult, LoaderItemUpdate, LoaderOutput, LoaderYieldInput, LoaderYieldResult,
  State as LoaderState,
};
use rspack_napi::js_tasks::Task;

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
  loader_context: Arc<LoaderContext<RunnerContext>>,
  input: &mut Option<LoaderYieldInput>,
) -> Result<Option<LoaderYieldResult>> {
  let input = input.take().expect("loader input is available");
  let chain = loader_context
    .current_chain()
    .expect("yield requires a loader chain");
  let current = loader_context.loader_index as usize;
  let mut start = chain.start();
  let mut end = chain.end();
  let mut strict = false;
  super::bridge::strict_loaders(self.owner, |references| {
    let Some(references) = references else {
      return;
    };
    let is_strict = |index: usize| {
      loader_context.loader_items[index]
        .query()
        .and_then(|query| query.strip_prefix("??"))
        .is_some_and(|ident| references.contains_key(ident))
    };
    strict = is_strict(current);
    start = current;
    end = current + 1;
    while start > chain.start() && is_strict(start - 1) == strict {
      start -= 1;
    }
    while end < chain.end() && is_strict(end) == strict {
      end += 1;
    }
  });

  // Skip a JavaScript execution span when no remaining loader needs pitching.
  if loader_context.state() == LoaderState::Pitching {
    let loaders_without_pitch = self.loaders_without_pitch.read().await;
    let start = loader_context.loader_index as usize;
    let needs_pitch = loader_context.loader_items[start..end]
      .iter()
      .enumerate()
      .any(|(offset, loader)| {
        !loader_context.loader_items[start + offset].pitch_executed()
          && !loaders_without_pitch.contains(loader.path().as_str())
      });
    if !needs_pitch {
      return Ok(Some(LoaderYieldResult::SkipPitch {
        input,
        range: start..end,
      }));
    }
  }

  if strict {
    drop(input.additional_data);
    let (sender, receiver) = tokio::sync::oneshot::channel();
    super::worker::dispatch(Task::new(
      StrictLoaderTask::new(
        loader_context,
        input.content,
        self.owner,
        start as u32,
        end as u32,
      ),
      sender,
      "Strict loader task was cancelled",
    ));
    let (new_cx, pitches) = receiver
      .await
      .expect("strict loader task returns its result")?;
    if !pitches.is_empty() {
      self.update_loaders_without_pitch(pitches).await;
    }
    return Ok(Some(convert_loader_result(new_cx)));
  }

  let additional_data = input
    .additional_data
    .and_then(|mut data| data.remove::<rspack_napi::ThreadsafeOneShotRef>());
  let mut context =
    JsLoaderContext::from_loader_context(&loader_context, input.content, additional_data);
  context.loader_chain_start = start as u32;
  context.loader_chain_end = end as u32;

  let runner = self.runner.lock().expect("should get lock").clone();
  let runner = runner
    .get_or_try_init(|| async {
      #[allow(clippy::unwrap_used)]
      let compiler_id = self.compiler_id.get().unwrap();
      self.runner_getter.call(compiler_id).await
    })
    .await
    .to_rspack_result()?;

  let new_cx = runner
    .call_async(context)
    .await
    .to_rspack_result()?
    .await
    .to_rspack_result()?;

  if loader_context.state() == LoaderState::Pitching {
    let list = collect_loaders_without_pitch(&loader_context, &new_cx);
    if !list.is_empty() {
      self.update_loaders_without_pitch(list).await;
    }
  }

  Ok(Some(convert_loader_result(new_cx)))
}

fn convert_loader_result(mut from: JsLoaderContext) -> LoaderYieldResult {
  let output = (|| {
    if let Some(error) = from.error {
      if let Some(diagnostic) = error.rust_diagnostic.as_ref() {
        return Err(diagnostic.error.clone());
      }
      return Err(error.with_parent_error_name("ModuleBuildError").into());
    }
    let content = match from.content {
      Either3::A(content) => Some(rspack_core::Content::String(content)),
      Either3::B(content) => Some(rspack_core::Content::Buffer(content.into())),
      Either3::C(_) => None,
    };
    let source_map = from
      .source_map
      .map(|buffer| rspack_core::rspack_sources::SourceMap::from_bytes(buffer.into()))
      .transpose()
      .to_rspack_result()?;
    let additional_data = from.additional_data.take().map(|data| {
      let mut additional = AdditionalData::default();
      additional.insert(data);
      additional
    });
    Ok(LoaderOutput {
      content,
      source_map,
      additional_data,
      loader_items: from
        .loader_items
        .into_iter()
        .map(|item| LoaderItemUpdate {
          normal_executed: item.normal_executed,
          pitch_executed: item.pitch_executed,
          filter_matched: item.filter_matched,
          data: item.data,
        })
        .collect(),
      loader_index: from.loader_index,
      parse_meta: from
        .parse_meta
        .into_iter()
        .map(|(key, value)| (key, Box::new(value) as _))
        .collect(),
    })
  })();
  LoaderYieldResult::Executed(LoaderExecutionResult {
    cacheable: from.cacheable,
    dependencies: from.dependencies.into(),
    added_dependencies: from.added_dependencies.into(),
    removed_dependencies: from.removed_dependencies.into(),
    output,
  })
}

pub(super) fn collect_loaders_without_pitch(
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
