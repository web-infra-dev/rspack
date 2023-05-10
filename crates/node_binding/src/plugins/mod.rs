mod loader;
use std::fmt::Debug;
use std::path::PathBuf;

use async_trait::async_trait;
pub use loader::JsLoaderResolver;
use napi::{Env, Result};
use rspack_binding_macros::js_fn_into_theadsafe_fn;
use rspack_core::{
  ChunkAssetArgs, NormalModuleBeforeResolveArgs, NormalModuleFactoryResolveForSchemeArgs,
  PluginNormalModuleFactoryBeforeResolveOutput, PluginNormalModuleFactoryResolveForSchemeOutput,
  ResourceData,
};
use rspack_error::internal_error;
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::NapiResultExt;

use crate::js_values::{
  BeforeResolveData, JsChunkAssetArgs, JsResourceData, SchemeAndJsResourceData,
};
use crate::{DisabledHooks, Hook, JsCompilation, JsHooks};

pub struct JsHooksAdapter {
  disabled_hooks: DisabledHooks,
  pub make_tsfn: ThreadsafeFunction<(), ()>,
  pub compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub process_assets_stage_additional_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_pre_process_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_additions_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_none_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_optimize_inline_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_summarize_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_optimize_hash_tsfn: ThreadsafeFunction<(), ()>,
  pub process_assets_stage_report_tsfn: ThreadsafeFunction<(), ()>,
  pub emit_tsfn: ThreadsafeFunction<(), ()>,
  pub after_emit_tsfn: ThreadsafeFunction<(), ()>,
  pub optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub before_compile_tsfn: ThreadsafeFunction<(), ()>,
  pub finish_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub chunk_asset_tsfn: ThreadsafeFunction<JsChunkAssetArgs, ()>,
  pub before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>>,
  pub context_module_before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>>,
  pub normal_module_factory_resolve_for_scheme:
    ThreadsafeFunction<SchemeAndJsResourceData, JsResourceData>,
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
    if self.is_hook_disabled(&Hook::Compilation) {
      return Ok(());
    }

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
    if self.is_hook_disabled(&Hook::ThisCompilation) {
      return Ok(());
    }

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

  async fn chunk_asset(&mut self, args: &ChunkAssetArgs) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::ChunkAsset) {
      return Ok(());
    }

    self
      .chunk_asset_tsfn
      .call(
        JsChunkAssetArgs::from(args),
        ThreadsafeFunctionCallMode::NonBlocking,
      )
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to chunk asset: {err}"))?
  }

  #[tracing::instrument(name = "js_hooks_adapter::make", skip_all)]
  async fn make(
    &self,
    _ctx: rspack_core::PluginContext,
    _compilation: &rspack_core::Compilation,
  ) -> rspack_core::PluginMakeHookOutput {
    if self.is_hook_disabled(&Hook::Make) {
      return Ok(());
    }

    // We don't need to expose `compilation` to Node as it's already been exposed via `compilation` hook
    self
      .make_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call make: {err}",))?
  }

  async fn before_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    self
      .before_resolve
      .call(args.clone().into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call this_compilation: {err}"))?
  }
  async fn context_module_before_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    self
      .context_module_before_resolve
      .call(args.clone().into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call this_compilation: {err}"))?
  }
  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &NormalModuleFactoryResolveForSchemeArgs,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    let res = self
      .normal_module_factory_resolve_for_scheme
      .call(args.clone().into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call this_compilation: {err}"))?;
    res.map(|res| {
      Some(ResourceData {
        resource: res.resource,
        resource_fragment: res.fragment,
        resource_path: PathBuf::from(res.path),
        resource_query: res.query,
        resource_description: None,
      })
    })
  }

  async fn process_assets_stage_additional(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    if self.is_hook_disabled(&Hook::ProcessAssetsStageAdditional) {
      return Ok(());
    }

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
    if self.is_hook_disabled(&Hook::ProcessAssetsStagePreProcess) {
      return Ok(());
    }

    self
      .process_assets_stage_pre_process_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage pre-process: {err}",))?
  }

  async fn process_assets_stage_additions(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    if self.is_hook_disabled(&Hook::ProcessAssetsStageAdditions) {
      return Ok(());
    }

    self
      .process_assets_stage_additions_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage additions: {err}",))?
  }

  async fn process_assets_stage_none(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    if self.is_hook_disabled(&Hook::ProcessAssetsStageNone) {
      return Ok(());
    }

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
    if self.is_hook_disabled(&Hook::ProcessAssetsStageOptimizeInline) {
      return Ok(());
    }

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
    if self.is_hook_disabled(&Hook::ProcessAssetsStageSummarize) {
      return Ok(());
    }

    // Directly calling hook processAssets without converting assets to JsAssets, instead, we use APIs to get `Source` lazily on the Node side.
    self
      .process_assets_stage_summarize_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage summarize: {err}",))?
  }

  async fn process_assets_stage_optimize_hash(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    if self.is_hook_disabled(&Hook::ProcessAssetsStageOptimizeHash) {
      return Ok(());
    }

    self
      .process_assets_stage_optimize_hash_tsfn
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
    if self.is_hook_disabled(&Hook::ProcessAssetsStageReport) {
      return Ok(());
    }
    // Directly calling hook processAssets without converting assets to JsAssets, instead, we use APIs to get `Source` lazily on the Node side.
    self
      .process_assets_stage_report_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call process assets stage report: {err}",))?
  }

  async fn optimize_modules(
    &mut self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeModules) {
      return Ok(());
    }
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });
    self
      .optimize_modules_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::Blocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call optimize modules: {err}"))?
  }

  async fn optimize_chunk_modules(
    &mut self,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeChunkModules) {
      return Ok(());
    }

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

  async fn before_compile(
    &mut self,
    // args: &mut rspack_core::CompilationArgs<'_>
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::BeforeCompile) {
      return Ok(());
    }

    self
      .before_compile_tsfn
      .call({}, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call before compile: {err}",))?
  }

  async fn finish_modules(
    &mut self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::FinishModules) {
      return Ok(());
    }

    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    self
      .finish_modules_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to finish modules: {err}"))?
  }

  async fn emit(&mut self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::Emit) {
      return Ok(());
    }

    self
      .emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call emit: {err}"))?
  }

  async fn after_emit(&mut self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterEmit) {
      return Ok(());
    }

    self
      .after_emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call after emit: {err}",))?
  }
}

impl JsHooksAdapter {
  pub fn from_js_hooks(env: Env, js_hooks: JsHooks, disabled_hooks: DisabledHooks) -> Result<Self> {
    let JsHooks {
      make,
      process_assets_stage_additional,
      process_assets_stage_pre_process,
      process_assets_stage_additions,
      process_assets_stage_none,
      process_assets_stage_optimize_inline,
      process_assets_stage_summarize,
      process_assets_stage_optimize_hash,
      process_assets_stage_report,
      this_compilation,
      compilation,
      emit,
      after_emit,
      optimize_modules,
      optimize_chunk_module,
      before_resolve,
      context_module_before_resolve,
      normal_module_factory_resolve_for_scheme,
      before_compile,
      finish_modules,
      chunk_asset,
    } = js_hooks;

    let process_assets_stage_additional_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_additional, env);
    let process_assets_stage_pre_process_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_pre_process, env);
    let process_assets_stage_additions_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_additions, env);
    let process_assets_stage_none_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_none, env);
    let process_assets_stage_optimize_inline_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_optimize_inline, env);
    let process_assets_stage_summarize_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_summarize, env);
    let process_assets_stage_optimize_hash_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_optimize_hash, env);
    let process_assets_stage_report_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(process_assets_stage_report, env);
    let emit_tsfn: ThreadsafeFunction<(), ()> = js_fn_into_theadsafe_fn!(emit, env);
    let after_emit_tsfn: ThreadsafeFunction<(), ()> = js_fn_into_theadsafe_fn!(after_emit, env);
    let this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_theadsafe_fn!(this_compilation, env);
    let compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_theadsafe_fn!(compilation, env);
    let make_tsfn: ThreadsafeFunction<(), ()> = js_fn_into_theadsafe_fn!(make, env);
    let optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_theadsafe_fn!(optimize_modules, env);
    let optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_theadsafe_fn!(optimize_chunk_module, env);
    let before_compile_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_theadsafe_fn!(before_compile, env);
    let finish_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_theadsafe_fn!(finish_modules, env);
    let context_module_before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>> =
      js_fn_into_theadsafe_fn!(context_module_before_resolve, env);
    let before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>> =
      js_fn_into_theadsafe_fn!(before_resolve, env);
    let normal_module_factory_resolve_for_scheme: ThreadsafeFunction<
      SchemeAndJsResourceData,
      JsResourceData,
    > = js_fn_into_theadsafe_fn!(normal_module_factory_resolve_for_scheme, env);
    let chunk_asset_tsfn: ThreadsafeFunction<JsChunkAssetArgs, ()> =
      js_fn_into_theadsafe_fn!(chunk_asset, env);

    Ok(JsHooksAdapter {
      disabled_hooks,
      make_tsfn,
      process_assets_stage_additional_tsfn,
      process_assets_stage_pre_process_tsfn,
      process_assets_stage_additions_tsfn,
      process_assets_stage_none_tsfn,
      process_assets_stage_optimize_inline_tsfn,
      process_assets_stage_summarize_tsfn,
      process_assets_stage_optimize_hash_tsfn,
      process_assets_stage_report_tsfn,
      compilation_tsfn,
      this_compilation_tsfn,
      emit_tsfn,
      after_emit_tsfn,
      optimize_modules_tsfn,
      optimize_chunk_modules_tsfn,
      before_compile_tsfn,
      before_resolve,
      context_module_before_resolve,
      normal_module_factory_resolve_for_scheme,
      finish_modules_tsfn,
      chunk_asset_tsfn,
    })
  }

  #[allow(clippy::unwrap_used)]
  fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.disabled_hooks.read().expect("").contains(hook)
  }
}
