mod interceptor;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
pub use interceptor::RegisterJsTaps;
use napi::{Env, Result};
use rspack_binding_values::JsAssetEmittedArgs;
use rspack_binding_values::JsChunkAssetArgs;
use rspack_binding_values::JsResolveForSchemeResult;
use rspack_core::PluginNormalModuleFactoryResolveForSchemeOutput;
use rspack_core::{
  ApplyContext, ChunkAssetArgs, CompilerOptions, NormalModuleAfterResolveArgs,
  NormalModuleAfterResolveCreateData, PluginContext,
};
use rspack_core::{BeforeResolveArgs, PluginNormalModuleFactoryAfterResolveOutput};
use rspack_core::{
  NormalModuleCreateData, PluginNormalModuleFactoryBeforeResolveOutput,
  PluginNormalModuleFactoryCreateModuleHookOutput, ResourceData,
};
use rspack_hook::Hook as _;

use self::interceptor::RegisterCompilationBuildModuleTaps;
use self::interceptor::RegisterCompilationStillValidModuleTaps;
use self::interceptor::RegisterCompilationSucceedModuleTaps;
use self::interceptor::{
  RegisterCompilationExecuteModuleTaps, RegisterCompilationProcessAssetsTaps,
  RegisterCompilationRuntimeModuleTaps, RegisterCompilerCompilationTaps, RegisterCompilerMakeTaps,
  RegisterCompilerShouldEmitTaps, RegisterCompilerThisCompilationTaps,
  RegisterNormalModuleFactoryBeforeResolveTaps,
};
use crate::{DisabledHooks, Hook, JsCompilation, JsHooks};

pub struct JsHooksAdapterInner {
  pub disabled_hooks: DisabledHooks,
  pub hooks: JsHooks,
}

#[derive(Clone)]
pub struct JsHooksAdapterPlugin {
  inner: Arc<JsHooksAdapterInner>,
  register_compiler_this_compilation_taps: RegisterCompilerThisCompilationTaps,
  register_compiler_compilation_taps: RegisterCompilerCompilationTaps,
  register_compiler_make_taps: RegisterCompilerMakeTaps,
  register_compiler_should_emit_taps: RegisterCompilerShouldEmitTaps,
  register_compilation_build_module_taps: RegisterCompilationBuildModuleTaps,
  register_compilation_still_valid_module_taps: RegisterCompilationStillValidModuleTaps,
  register_compilation_succeed_module_taps: RegisterCompilationSucceedModuleTaps,
  register_compilation_execute_module_taps: RegisterCompilationExecuteModuleTaps,
  register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps,
  register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps,
  register_normal_module_factory_before_resolve_taps: RegisterNormalModuleFactoryBeforeResolveTaps,
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
      .this_compilation
      .intercept(self.register_compiler_this_compilation_taps.clone());
    ctx
      .context
      .compiler_hooks
      .compilation
      .intercept(self.register_compiler_compilation_taps.clone());
    ctx
      .context
      .compiler_hooks
      .make
      .intercept(self.register_compiler_make_taps.clone());
    ctx
      .context
      .compiler_hooks
      .should_emit
      .intercept(self.register_compiler_should_emit_taps.clone());
    ctx
      .context
      .compilation_hooks
      .build_module
      .intercept(self.register_compilation_build_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .still_valid_module
      .intercept(self.register_compilation_still_valid_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .succeed_module
      .intercept(self.register_compilation_succeed_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .execute_module
      .intercept(self.register_compilation_execute_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .runtime_module
      .intercept(self.register_compilation_runtime_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .process_assets
      .intercept(self.register_compilation_process_assets_taps.clone());
    ctx
      .context
      .normal_module_factory_hooks
      .before_resolve
      .intercept(
        self
          .register_normal_module_factory_before_resolve_taps
          .clone(),
      );
    Ok(())
  }

  async fn chunk_asset(&self, args: &ChunkAssetArgs) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::ChunkAsset) {
      return Ok(());
    }

    self
      .hooks
      .chunk_asset
      .call(JsChunkAssetArgs::from(args))
      .await
  }

  async fn after_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut NormalModuleAfterResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryAfterResolveOutput {
    if self.is_hook_disabled(&Hook::AfterResolve) {
      return Ok(None);
    }

    match self.hooks.after_resolve.call((&*args).into()).await {
      Ok((ret, resolve_data)) => {
        if let (Some(resolve_data), Some(create_data)) = (resolve_data, &args.create_data) {
          fn override_resource(origin_data: &ResourceData, new_resource: String) -> ResourceData {
            let mut resource_data = origin_data.clone();
            let origin_resource_path = origin_data.resource_path.to_string_lossy().to_string();
            resource_data.resource_path = new_resource.clone().into();
            resource_data.resource = resource_data
              .resource
              .replace(&origin_resource_path, &new_resource);

            resource_data
          }

          let request = resolve_data.request;
          let user_request = resolve_data.user_request;
          let resource = override_resource(&create_data.resource, resolve_data.resource);

          args.create_data = Some(NormalModuleAfterResolveCreateData {
            request,
            user_request,
            resource,
          });
        }

        Ok(ret)
      }
      Err(err) => Err(err),
    }
  }

  async fn context_module_before_resolve(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut BeforeResolveArgs,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    if self.is_hook_disabled(&Hook::ContextModuleFactoryBeforeResolve) {
      return Ok(None);
    }
    self
      .hooks
      .context_module_factory_before_resolve
      .call(args.clone().into())
      .await
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
      .hooks
      .context_module_factory_after_resolve
      .call((&*args).into())
      .await
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
      .hooks
      .normal_module_factory_create_module
      .call(args.into())
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
      .hooks
      .normal_module_factory_resolve_for_scheme
      .call(args.into())
      .await;
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
    self.hooks.after_process_assets.call(()).await
  }

  async fn optimize_modules(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeModules) {
      return Ok(());
    }
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };
    self.hooks.optimize_modules.call(compilation).await
  }

  async fn after_optimize_modules(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterOptimizeModules) {
      return Ok(());
    }
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };
    self.hooks.after_optimize_modules.call(compilation).await
  }

  async fn optimize_tree(
    &self,
    _compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeTree) {
      return Ok(());
    }
    self.hooks.optimize_tree.call(()).await
  }

  async fn optimize_chunk_modules(
    &self,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::OptimizeChunkModules) {
      return Ok(());
    }

    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(args.compilation) };

    self.hooks.optimize_chunk_modules.call(compilation).await
  }

  async fn finish_make(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::FinishMake) {
      return Ok(());
    }

    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.hooks.finish_make.call(compilation).await
  }

  async fn finish_modules(
    &self,
    compilation: &mut rspack_core::Compilation,
  ) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::FinishModules) {
      return Ok(());
    }

    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.hooks.finish_modules.call(compilation).await
  }

  async fn emit(&self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::Emit) {
      return Ok(());
    }

    self.hooks.emit.call(()).await
  }

  async fn asset_emitted(&self, args: &rspack_core::AssetEmittedArgs) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AssetEmitted) {
      return Ok(());
    }

    let args: JsAssetEmittedArgs = args.into();
    self.hooks.asset_emitted.call(args).await
  }

  async fn after_emit(&self, _: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    if self.is_hook_disabled(&Hook::AfterEmit) {
      return Ok(());
    }

    self.hooks.after_emit.call(()).await
  }
}

impl JsHooksAdapterPlugin {
  pub fn from_js_hooks(
    _env: Env,
    js_hooks: JsHooks,
    disabled_hooks: DisabledHooks,
    register_js_taps: RegisterJsTaps,
  ) -> Result<Self> {
    Ok(JsHooksAdapterPlugin {
      register_compiler_this_compilation_taps: RegisterCompilerThisCompilationTaps::new(
        register_js_taps.register_compiler_this_compilation_taps,
      ),
      register_compiler_compilation_taps: RegisterCompilerCompilationTaps::new(
        register_js_taps.register_compiler_compilation_taps,
      ),
      register_compiler_make_taps: RegisterCompilerMakeTaps::new(
        register_js_taps.register_compiler_make_taps,
      ),
      register_compiler_should_emit_taps: RegisterCompilerShouldEmitTaps::new(
        register_js_taps.register_compiler_should_emit_taps,
      ),
      register_compilation_build_module_taps: RegisterCompilationBuildModuleTaps::new(
        register_js_taps.register_compilation_build_module_taps,
      ),
      register_compilation_still_valid_module_taps: RegisterCompilationStillValidModuleTaps::new(
        register_js_taps.register_compilation_still_valid_module_taps,
      ),
      register_compilation_succeed_module_taps: RegisterCompilationSucceedModuleTaps::new(
        register_js_taps.register_compilation_succeed_module_taps,
      ),
      register_compilation_execute_module_taps: RegisterCompilationExecuteModuleTaps::new(
        register_js_taps.register_compilation_execute_module_taps,
      ),
      register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps::new(
        register_js_taps.register_compilation_runtime_module_taps,
      ),
      register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps::new(
        register_js_taps.register_compilation_process_assets_taps,
      ),
      register_normal_module_factory_before_resolve_taps:
        RegisterNormalModuleFactoryBeforeResolveTaps::new(
          register_js_taps.register_normal_module_factory_before_resolve_taps,
        ),
      inner: Arc::new(JsHooksAdapterInner {
        disabled_hooks,
        hooks: js_hooks,
      }),
    })
  }

  fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.disabled_hooks.is_hook_disabled(hook)
  }

  pub fn set_disabled_hooks(&self, hooks: Vec<String>) {
    self.disabled_hooks.set_disabled_hooks(hooks)
  }
}
