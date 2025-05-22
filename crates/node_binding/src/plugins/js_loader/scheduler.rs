use napi::Either;
use rspack_core::{
  diagnostics::CapturedLoaderError, AdditionalData, LoaderContext, NormalModuleLoaderShouldYield,
  NormalModuleLoaderStartYielding, RunnerContext, BUILTIN_LOADER_PREFIX,
};
use rspack_error::{miette::IntoDiagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_runner::State as LoaderState;

use super::{JsLoaderContext, JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[plugin_hook(NormalModuleLoaderShouldYield for JsLoaderRspackPlugin, tracing=false)]
pub(crate) async fn loader_should_yield(
  &self,
  loader_context: &LoaderContext<RunnerContext>,
) -> Result<Option<bool>> {
  match loader_context.state() {
    s @ LoaderState::Init | s @ LoaderState::ProcessResource | s @ LoaderState::Finished => {
      panic!("Unexpected loader runner state: {s:?}")
    }
    LoaderState::Pitching | LoaderState::Normal => Ok(Some(
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
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<()> {
  let read_guard = self.runner.read().await;
  match &*read_guard {
    Some(runner) => {
      let new_cx = runner
        .call_async(loader_context.try_into()?)
        .await
        .into_diagnostic()?
        .await
        .into_diagnostic()?;
      drop(read_guard);

      merge_loader_context(loader_context, new_cx)?;
    }
    None => {
      drop(read_guard);

      {
        let mut write_guard = self.runner.write().await;
        #[allow(clippy::unwrap_used)]
        let compiler_id = self.compiler_id.get().unwrap();
        #[allow(clippy::unwrap_used)]
        let runner = self
          .runner_getter
          .call(compiler_id)
          .await
          .into_diagnostic()?;
        *write_guard = Some(runner);
      };

      let read_guard = self.runner.read().await;
      #[allow(clippy::unwrap_used)]
      let new_cx = read_guard
        .as_ref()
        .unwrap()
        .call_async(loader_context.try_into()?)
        .await
        .into_diagnostic()?
        .await
        .into_diagnostic()?;
      drop(read_guard);

      merge_loader_context(loader_context, new_cx)?;
    }
  };
  Ok(())
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  if let Some(error) = from.error {
    return Err(
      CapturedLoaderError::new(
        error.message,
        error.stack,
        error.hide_stack,
        from.file_dependencies,
        from.context_dependencies,
        from.missing_dependencies,
        from.build_dependencies,
        from.cacheable,
      )
      .into(),
    );
  }

  to.cacheable = from.cacheable;
  to.file_dependencies = from.file_dependencies.into_iter().map(Into::into).collect();
  to.context_dependencies = from
    .context_dependencies
    .into_iter()
    .map(Into::into)
    .collect();
  to.missing_dependencies = from
    .missing_dependencies
    .into_iter()
    .map(Into::into)
    .collect();
  to.build_dependencies = from
    .build_dependencies
    .into_iter()
    .map(Into::into)
    .collect();

  let content = match from.content {
    Either::A(_) => None,
    Either::B(c) => {
      // perf: Ignore UTF-8 check when JavaScript passed in an UTF-8 encoded value
      let content = if let Some(utf8_hint) = from.utf8_hint
        && utf8_hint
      {
        rspack_core::Content::from(
          // SAFETY: UTF-8 passed from JavaScript loader runner should ensure it does not pass non-UTF-8 encoded sequence when `utf_hint` is set to `true`. This invariant should be followed on the JavaScript side.
          unsafe { String::from_utf8_unchecked(c.into()) },
        )
      } else {
        rspack_core::Content::from(Into::<Vec<u8>>::into(c))
      };

      Some(content)
    }
  };
  let source_map = from
    .source_map
    .as_ref()
    .map(|s| {
      rspack_core::rspack_sources::SourceMap::from_json(
        // SAFETY: `sourceMap` is serialized by JavaScript from a JSON object. This is an invariant should be followed on the JavaScript side.
        unsafe { str::from_utf8_unchecked(s) },
      )
    })
    .transpose()
    .to_rspack_result()?;
  let additional_data = from.additional_data.take().map(|data| {
    let mut additional = AdditionalData::default();
    additional.insert(data);
    additional
  });
  to.__finish_with((content, source_map, additional_data));

  // update loader status
  to.loader_items = to
    .loader_items
    .drain(..)
    .zip(from.loader_items.drain(..))
    .map(|(mut to, from)| {
      if from.normal_executed {
        to.set_normal_executed()
      }
      if from.pitch_executed {
        to.set_pitch_executed()
      }
      to.set_data(from.data);
      // JS loader should always be considered as finished
      to.set_finish_called();
      to
    })
    .collect();
  to.loader_index = from.loader_index;
  to.parse_meta = from.parse_meta.into_iter().collect();

  Ok(())
}
