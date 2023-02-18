use std::fmt::Debug;

use async_trait::async_trait;
use napi::{Env, NapiRaw, Result};
use rspack_error::internal_error;
use rspack_napi_utils::NapiResultExt;

use crate::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use crate::{JsCompilation, JsHooks};

pub struct JsHooksAdapter {
  pub make_tsfn: ThreadsafeFunction<(), ()>,
  pub compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub process_assets_stage_additional_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_pre_process_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_none_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_optimize_inline_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_summarize_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_report_tsfn: ThreadsafeFunction<(), ()>,
  pub emit_tsfn: ThreadsafeFunction<(), ()>,
  pub after_emit_tsfn: ThreadsafeFunction<(), ()>,
  pub optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
}

impl Debug for JsHooksAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "rspack_plugin_js_hooks_adapter")
  }
}

#[async_trait]
impl rspack_core::Plugin for JsHooksAdapter {
  fn name(&self) -> &'static str {
    "rspack_plugin_js_hooks_adapter"
  }

  async fn compilation(
    &mut self,
    args: rspack_core::CompilationArgs<'_>,
  ) -> rspack_core::PluginCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        args.compilation,
      )
    });

    self
      .compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call compilation: {err}"))?
  }

  async fn this_compilation(
    &mut self,
    args: rspack_core::ThisCompilationArgs<'_>,
  ) -> rspack_core::PluginThisCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        args.this_compilation,
      )
    });

    self
      .this_compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call this_compilation: {err}"))?
  }

  #[tracing::instrument(name = "js_hooks_adapter::make", skip_all)]
  async fn make(
    &self,
    _ctx: rspack_core::PluginContext,
    _compilation: &rspack_core::Compilation,
  ) -> rspack_core::PluginMakeHookOutput {
    // We don't need to expose `compilation` to Node as it's already been exposed via `compilation` hook
    self
      .make_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call make: {err}",))?
  }

  async fn process_assets_stage_additional(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    self
      .process_assets_stage_additional_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage additional: {err}",))?
  }

  async fn process_assets_stage_pre_process(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    self
      .process_assets_stage_pre_process_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage pre-process: {err}",))?
  }

  async fn process_assets_stage_none(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    self
      .process_assets_stage_none_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets: {err}",))?
  }

  async fn process_assets_stage_optimize_inline(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    self
      .process_assets_stage_optimize_inline_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| {
        internal_error!("Failed to call process assets stage optimize inline: {err}",)
      })?
  }

  async fn process_assets_stage_summarize(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    // Directly calling hook processAssets without converting assets to JsAssets, instead, we use APIs to get `Source` lazily on the Node side.
    self
      .process_assets_stage_summarize_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage summarize: {err}",))?
  }

  async fn process_assets_stage_report(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    // Directly calling hook processAssets without converting assets to JsAssets, instead, we use APIs to get `Source` lazily on the Node side.
    self
      .process_assets_stage_report_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage report: {err}",))?
  }

  async fn emit(&mut self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    self
      .emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call emit: {err}"))?
  }

  async fn after_emit(&mut self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    self
      .after_emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call after emit: {err}",))?
  }

  async fn optimize_chunk_modules(
    &mut self,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        args.compilation,
      )
    });

    self
      .optimize_chunk_modules_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to compilation: {err}"))?
  }
}

impl JsHooksAdapter {
  pub fn from_js_hooks(env: Env, js_hooks: JsHooks) -> Result<Self> {
    let JsHooks {
      make,
      process_assets_stage_additional,
      process_assets_stage_pre_process,
      process_assets_stage_none,
      process_assets_stage_optimize_inline,
      process_assets_stage_summarize,
      process_assets_stage_report,
      this_compilation,
      compilation,
      emit,
      after_emit,
      optimize_chunk_module,
    } = js_hooks;

    // *Note* that the order of the creation of threadsafe function is important. There is a queue of threadsafe calls for each tsfn:
    // For example:
    // tsfn1: [call-in-js-task1, call-in-js-task2]
    // tsfn2: [call-in-js-task3, call-in-js-task4]
    // If the tsfn1 is created before tsfn2, and task1 is created(via `tsfn.call`) before task2(single tsfn level),
    // and *if these tasks are created in the same tick*, tasks will be called on main thread in the order of `task1` `task2` `task3` `task4`
    //
    // In practice:
    // The creation of callback `this_compilation` is placed before the callback `compilation` because we want the JS hooks `this_compilation` to be called before the JS hooks `compilation`.

    macro_rules! create_hook_tsfn {
      ($js_cb: expr) => {{
        let cb = unsafe { $js_cb.raw() };

        let mut tsfn: ThreadsafeFunction<_, _> =
          ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let (ctx, resolver) = ctx.split_into_parts();

            let env = ctx.env;
            let cb = ctx.callback;
            let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) };

            resolver.resolve::<()>(result, |_| Ok(()))
          })?;

        // See the comment in `threadsafe_function.rs`
        tsfn.unref(&env)?;
        tsfn
      }};
    }

    let process_assets_stage_additional_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_additional);
    let process_assets_stage_pre_process_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_pre_process);
    let process_assets_stage_none_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_none);
    let process_assets_stage_optimize_inline_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_optimize_inline);
    let process_assets_stage_summarize_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_summarize);
    let process_assets_stage_report_tsfn: ThreadsafeFunction<(), ()> =
      create_hook_tsfn!(process_assets_stage_report);
    let emit_tsfn: ThreadsafeFunction<(), ()> = create_hook_tsfn!(emit);
    let after_emit_tsfn: ThreadsafeFunction<(), ()> = create_hook_tsfn!(after_emit);
    let this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      create_hook_tsfn!(this_compilation);
    let compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> = create_hook_tsfn!(compilation);
    let make_tsfn: ThreadsafeFunction<(), ()> = create_hook_tsfn!(make);
    let optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      create_hook_tsfn!(optimize_chunk_module);

    Ok(JsHooksAdapter {
      make_tsfn,
      process_assets_stage_additional_tsfn,
      process_assets_stage_pre_process_tsfn,
      process_assets_stage_none_tsfn,
      process_assets_stage_optimize_inline_tsfn,
      process_assets_stage_summarize_tsfn,
      process_assets_stage_report_tsfn,
      compilation_tsfn,
      this_compilation_tsfn,
      emit_tsfn,
      after_emit_tsfn,
      optimize_chunk_modules_tsfn,
    })
  }
}
