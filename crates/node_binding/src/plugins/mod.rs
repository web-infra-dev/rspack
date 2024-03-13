mod interceptor;
mod loader;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
pub use interceptor::RegisterJsTaps;
use napi::{Env, Result};
use rspack_binding_macros::js_fn_into_threadsafe_fn;
use rspack_binding_values::{
  AfterResolveData, JsChunk, JsChunkAssetArgs, JsModule, JsRuntimeModule, JsRuntimeModuleArg,
  ToJsCompatSource,
};
use rspack_binding_values::{BeforeResolveData, JsAssetEmittedArgs, ToJsModule};
use rspack_binding_values::{CreateModuleData, JsBuildTimeExecutionOption, JsExecuteModuleArg};
use rspack_binding_values::{JsResolveForSchemeInput, JsResolveForSchemeResult};
use rspack_core::rspack_sources::Source;
use rspack_core::{
  ApplyContext, BuildTimeExecutionOption, Chunk, ChunkAssetArgs, CompilerOptions, ModuleIdentifier,
  NormalModuleAfterResolveArgs, PluginContext, RuntimeModule,
};
use rspack_core::{NormalModuleBeforeResolveArgs, PluginNormalModuleFactoryAfterResolveOutput};
use rspack_core::{
  NormalModuleCreateData, PluginNormalModuleFactoryBeforeResolveOutput,
  PluginNormalModuleFactoryCreateModuleHookOutput, ResourceData,
};
use rspack_core::{PluginNormalModuleFactoryResolveForSchemeOutput, PluginShouldEmitHookOutput};
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::NapiResultExt;

pub use self::loader::JsLoaderResolver;
use crate::{DisabledHooks, Hook, JsCompilation, JsHooks};

pub struct JsHooksAdapterInner {
  pub disabled_hooks: DisabledHooks,
  pub this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub after_process_assets_tsfn: ThreadsafeFunction<(), ()>,
  pub emit_tsfn: ThreadsafeFunction<(), ()>,
  pub asset_emitted_tsfn: ThreadsafeFunction<JsAssetEmittedArgs, ()>,
  pub should_emit_tsfn: ThreadsafeFunction<JsCompilation, Option<bool>>,
  pub after_emit_tsfn: ThreadsafeFunction<(), ()>,
  pub optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub after_optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub optimize_tree_tsfn: ThreadsafeFunction<(), ()>,
  pub optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub before_compile_tsfn: ThreadsafeFunction<(), ()>,
  pub after_compile_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub finish_modules_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub finish_make_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub build_module_tsfn: ThreadsafeFunction<JsModule, ()>, // TODO
  pub chunk_asset_tsfn: ThreadsafeFunction<JsChunkAssetArgs, ()>,
  pub before_resolve: ThreadsafeFunction<BeforeResolveData, (Option<bool>, BeforeResolveData)>,
  pub after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>>,
  pub context_module_factory_before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>>,
  pub context_module_factory_after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>>,
  pub normal_module_factory_create_module: ThreadsafeFunction<CreateModuleData, ()>,
  pub normal_module_factory_resolve_for_scheme:
    ThreadsafeFunction<JsResolveForSchemeInput, JsResolveForSchemeResult>,
  pub succeed_module_tsfn: ThreadsafeFunction<JsModule, ()>,
  pub still_valid_module_tsfn: ThreadsafeFunction<JsModule, ()>,
  pub execute_module_tsfn: ThreadsafeFunction<JsExecuteModuleArg, ()>,
  pub runtime_module_tsfn: ThreadsafeFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>>,
}

#[derive(Clone)]
pub struct JsHooksAdapterPlugin {
  inner: Arc<JsHooksAdapterInner>,
  interceptor: Box<RegisterJsTaps>,
}

impl fmt::Debug for JsHooksAdapterPlugin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "rspack_plugin_js_hooks_adapter")
  }
}

// TODO: remove deref
impl std::ops::Deref for JsHooksAdapterPlugin {
  type Target = JsHooksAdapterInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

#[async_trait]
impl rspack_core::Plugin for JsHooksAdapterPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsHooksAdapterPlugin"
  }

  #[tracing::instrument(name = "js_hooks_adapter::apply", skip_all)]
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .intercept(self.interceptor.clone());
    ctx
      .context
      .compiler_hooks
      .make
      .intercept(self.interceptor.clone());
    ctx
      .context
      .compilation_hooks
      .process_assets
      .intercept(self.interceptor.clone());
    Ok(())
  }

  async fn this_compilation(
    &self,
    args: rspack_core::ThisCompilationArgs<'_>,
    _params: &rspack_core::CompilationParams,
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
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"))
  }

  async fn chunk_asset(&self, args: &ChunkAssetArgs) -> rspack_error::Result<()> {
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
      .unwrap_or_else(|err| panic!("Failed to chunk asset: {err}"))
  }

  async fn before_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    if self.is_hook_disabled(&Hook::BeforeResolve) {
      return Ok(None);
    }
    match self
      .before_resolve
      .call(args.clone().into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"))
    {
      Ok((ret, resolve_data)) => {
        args.request = resolve_data.request;
        args.context = resolve_data.context;
        Ok(ret)
      }
      Err(err) => Err(err),
    }
  }

  async fn after_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleAfterResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryAfterResolveOutput {
    if self.is_hook_disabled(&Hook::AfterResolve) {
      return Ok(None);
    }
    self
      .after_resolve
      .call((&*args).into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"))
  }

  async fn context_module_before_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleBeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    if self.is_hook_disabled(&Hook::ContextModuleFactoryBeforeResolve) {
      return Ok(None);
    }
    self
      .context_module_factory_before_resolve
      .call(args.clone().into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"))
  }

  async fn context_module_after_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleAfterResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    if self.is_hook_disabled(&Hook::ContextModuleFactoryAfterResolve) {
      return Ok(None);
    }
    self
      .context_module_factory_after_resolve
      .call((&*args).into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"))
  }

  async fn normal_module_factory_create_module(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleCreateData<'_>,
  ) -> PluginNormalModuleFactoryCreateModuleHookOutput {
    if self.is_hook_disabled(&Hook::NormalModuleFactoryCreateModule) {
      return Ok(None);
    }
    self
      .normal_module_factory_create_module
      .call(args.into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map(|_| None)
      .map_err(|err| panic!("Failed to call this_compilation: {err}"))
  }

  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: rspack_core::PluginContext,
    args: ResourceData,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    if self.is_hook_disabled(&Hook::NormalModuleFactoryResolveForScheme) {
      return Ok((args, false));
    }
    let res = self
      .normal_module_factory_resolve_for_scheme
      .call(args.into(), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call this_compilation: {err}"));
    res.map(|res| {
      let JsResolveForSchemeResult {
        resource_data,
        stop,
      } = res;
      (
        ResourceData::new(resource_data.resource, PathBuf::from(resource_data.path))
          .query_optional(resource_data.query)
          .fragment_optional(resource_data.fragment),
        stop,
      )
    })
  }

  async fn after_process_assets(
    &self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    if self.is_hook_disabled(&Hook::AfterProcessAssets) {
      return Ok(());
    }
    self
      .after_process_assets_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call process assets stage report: {err}"))
  }

  async fn optimize_modules(
    &self,
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
      .unwrap_or_else(|err| panic!("Failed to call optimize modules: {err}"))
  }

  async fn after_optimize_modules(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterOptimizeModules) {
      return Ok(());
    }
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });
    self
      .after_optimize_modules_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::Blocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call optimize modules: {err}"))
  }

  async fn optimize_tree(
    &self,
    _compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeTree) {
      return Ok(());
    }
    self
      .optimize_tree_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call optimize tree: {err}"))
  }

  async fn optimize_chunk_modules(
    &self,
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
      .unwrap_or_else(|err| panic!("Failed to compilation: {err}"))
  }

  async fn before_compile(
    &self,
    _params: &rspack_core::CompilationParams,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::BeforeCompile) {
      return Ok(());
    }

    self
      .before_compile_tsfn
      .call({}, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call before compile: {err}"))
  }

  async fn after_compile(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterCompile) {
      return Ok(());
    }

    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    self
      .after_compile_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call after compile: {err}"))
  }

  async fn finish_make(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::FinishMake) {
      return Ok(());
    }

    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    self
      .finish_make_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call finish make: {err}"))
  }

  async fn build_module(&self, module: &mut dyn rspack_core::Module) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::BuildModule) {
      return Ok(());
    }

    self
      .build_module_tsfn
      .call(
        module.to_js_module().expect("Convert to js_module failed."),
        ThreadsafeFunctionCallMode::NonBlocking,
      )
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call build module: {err}"))
  }

  async fn finish_modules(
    &self,
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
      .unwrap_or_else(|err| panic!("Failed to finish modules: {err}"))
  }

  async fn emit(&self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::Emit) {
      return Ok(());
    }

    self
      .emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call emit: {err}"))
  }

  async fn asset_emitted(&self, args: &rspack_core::AssetEmittedArgs) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AssetEmitted) {
      return Ok(());
    }

    let args: JsAssetEmittedArgs = args.into();
    self
      .asset_emitted_tsfn
      .call(args, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call asset emitted: {err}"))
  }

  async fn should_emit(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> PluginShouldEmitHookOutput {
    if self.is_hook_disabled(&Hook::ShouldEmit) {
      return Ok(None);
    }

    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    let res = self
      .should_emit_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await;
    res.unwrap_or_else(|err| panic!("Failed to call should emit: {err}"))
  }

  async fn after_emit(&self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterEmit) {
      return Ok(());
    }

    self
      .after_emit_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call after emit: {err}"))
  }

  async fn succeed_module(&self, args: &dyn rspack_core::Module) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::SucceedModule) {
      return Ok(());
    }
    let js_module = args
      .to_js_module()
      .expect("Failed to convert module to JsModule");
    self
      .succeed_module_tsfn
      .call(js_module, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call succeed_module hook: {err}"))
  }

  async fn still_valid_module(&self, args: &dyn rspack_core::Module) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::StillValidModule) {
      return Ok(());
    }

    self
      .still_valid_module_tsfn
      .call(
        args.to_js_module().expect("Convert to js_module failed."),
        ThreadsafeFunctionCallMode::NonBlocking,
      )
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call still_valid_module hook: {err}"))
  }

  fn execute_module(
    &self,
    entry: ModuleIdentifier,
    request: &str,
    options: &BuildTimeExecutionOption,
    runtime_modules: Vec<ModuleIdentifier>,
    codegen_results: &rspack_core::CodeGenerationResults,
    id: u32,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::ExecuteModule) {
      return Ok(());
    }

    self
      .execute_module_tsfn
      .call(
        JsExecuteModuleArg {
          entry: entry.to_string(),
          request: request.into(),
          options: JsBuildTimeExecutionOption {
            public_path: options.public_path.clone(),
            base_uri: options.base_uri.clone(),
          },
          runtime_modules: runtime_modules
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
          codegen_results: codegen_results.clone().into(),
          id,
        },
        ThreadsafeFunctionCallMode::NonBlocking,
      )
      .into_rspack_result()?
      .blocking_recv()
      .unwrap_or_else(|recv_err| panic!("{}", recv_err.to_string()))
  }

  async fn runtime_module(
    &self,
    module: &mut dyn RuntimeModule,
    source: Arc<dyn Source>,
    chunk: &Chunk,
  ) -> rspack_error::Result<Option<String>> {
    if self.is_hook_disabled(&Hook::RuntimeModule) {
      return Ok(None);
    }

    self
      .runtime_module_tsfn
      .call(
        JsRuntimeModuleArg {
          module: JsRuntimeModule {
            source: Some(
              source
                .to_js_compat_source()
                .unwrap_or_else(|err| panic!("Failed to generate runtime module source: {err}")),
            ),
            module_identifier: module.identifier().to_string(),
            constructor_name: module.get_constructor_name(),
            name: module
              .identifier()
              .to_string()
              .replace("webpack/runtime/", ""),
          },
          chunk: JsChunk::from(chunk),
        },
        ThreadsafeFunctionCallMode::NonBlocking,
      )
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call runtime module hook: {err}"))
      .map(|r| {
        r.and_then(|s| s.source).map(|s| {
          std::str::from_utf8(&s.source)
            .unwrap_or_else(|err| panic!("Failed to covert buffer to utf-8 string: {err}"))
            .to_string()
        })
      })
  }
}

impl JsHooksAdapterPlugin {
  pub fn from_js_hooks(
    env: Env,
    js_hooks: JsHooks,
    disabled_hooks: DisabledHooks,
    interceptor: RegisterJsTaps,
  ) -> Result<Self> {
    let JsHooks {
      after_process_assets,
      this_compilation,
      should_emit,
      emit,
      asset_emitted,
      after_emit,
      optimize_modules,
      after_optimize_modules,
      optimize_tree,
      optimize_chunk_modules,
      before_resolve,
      after_resolve,
      context_module_factory_before_resolve,
      context_module_factory_after_resolve,
      normal_module_factory_create_module,
      normal_module_factory_resolve_for_scheme,
      before_compile,
      after_compile,
      finish_modules,
      finish_make,
      build_module,
      chunk_asset,
      succeed_module,
      still_valid_module,
      execute_module,
      runtime_module,
    } = js_hooks;

    let after_process_assets_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_threadsafe_fn!(after_process_assets, env);
    let emit_tsfn: ThreadsafeFunction<(), ()> = js_fn_into_threadsafe_fn!(emit, env);
    let should_emit_tsfn: ThreadsafeFunction<JsCompilation, Option<bool>> =
      js_fn_into_threadsafe_fn!(should_emit, env);
    let asset_emitted_tsfn: ThreadsafeFunction<JsAssetEmittedArgs, ()> =
      js_fn_into_threadsafe_fn!(asset_emitted, env);
    let after_emit_tsfn: ThreadsafeFunction<(), ()> = js_fn_into_threadsafe_fn!(after_emit, env);
    let this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(this_compilation, env);
    let optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(optimize_modules, env);
    let after_optimize_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(after_optimize_modules, env);
    let optimize_tree_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_threadsafe_fn!(optimize_tree, env);
    let optimize_chunk_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(optimize_chunk_modules, env);
    let before_compile_tsfn: ThreadsafeFunction<(), ()> =
      js_fn_into_threadsafe_fn!(before_compile, env);
    let after_compile_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(after_compile, env);
    let finish_make_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(finish_make, env);
    let build_module_tsfn: ThreadsafeFunction<JsModule, ()> =
      js_fn_into_threadsafe_fn!(build_module, env);
    let finish_modules_tsfn: ThreadsafeFunction<JsCompilation, ()> =
      js_fn_into_threadsafe_fn!(finish_modules, env);
    let context_module_factory_before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>> =
      js_fn_into_threadsafe_fn!(context_module_factory_before_resolve, env);
    let context_module_factory_after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>> =
      js_fn_into_threadsafe_fn!(context_module_factory_after_resolve, env);
    let before_resolve: ThreadsafeFunction<BeforeResolveData, (Option<bool>, BeforeResolveData)> =
      js_fn_into_threadsafe_fn!(before_resolve, env);
    let after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>> =
      js_fn_into_threadsafe_fn!(after_resolve, env);
    let normal_module_factory_create_module: ThreadsafeFunction<CreateModuleData, ()> =
      js_fn_into_threadsafe_fn!(normal_module_factory_create_module, env);
    let normal_module_factory_resolve_for_scheme: ThreadsafeFunction<
      JsResolveForSchemeInput,
      JsResolveForSchemeResult,
    > = js_fn_into_threadsafe_fn!(normal_module_factory_resolve_for_scheme, env);
    let chunk_asset_tsfn: ThreadsafeFunction<JsChunkAssetArgs, ()> =
      js_fn_into_threadsafe_fn!(chunk_asset, env);
    let succeed_module_tsfn: ThreadsafeFunction<JsModule, ()> =
      js_fn_into_threadsafe_fn!(succeed_module, env);
    let still_valid_module_tsfn: ThreadsafeFunction<JsModule, ()> =
      js_fn_into_threadsafe_fn!(still_valid_module, env);
    let execute_module_tsfn: ThreadsafeFunction<JsExecuteModuleArg, ()> =
      js_fn_into_threadsafe_fn!(execute_module, env);
    let runtime_module_tsfn: ThreadsafeFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>> =
      js_fn_into_threadsafe_fn!(runtime_module, env);

    Ok(JsHooksAdapterPlugin {
      interceptor: Box::new(interceptor),
      inner: Arc::new(JsHooksAdapterInner {
        disabled_hooks,
        after_process_assets_tsfn,
        this_compilation_tsfn,
        should_emit_tsfn,
        emit_tsfn,
        asset_emitted_tsfn,
        after_emit_tsfn,
        optimize_modules_tsfn,
        after_optimize_modules_tsfn,
        optimize_tree_tsfn,
        optimize_chunk_modules_tsfn,
        before_compile_tsfn,
        after_compile_tsfn,
        before_resolve,
        context_module_factory_before_resolve,
        context_module_factory_after_resolve,
        normal_module_factory_create_module,
        normal_module_factory_resolve_for_scheme,
        finish_modules_tsfn,
        finish_make_tsfn,
        build_module_tsfn,
        chunk_asset_tsfn,
        after_resolve,
        succeed_module_tsfn,
        still_valid_module_tsfn,
        execute_module_tsfn,
        runtime_module_tsfn,
      }),
    })
  }

  fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.disabled_hooks.is_hook_disabled(hook)
  }

  pub fn set_disabled_hooks(&self, hooks: Vec<String>) -> Result<()> {
    self.disabled_hooks.set_disabled_hooks(hooks)
  }
}
