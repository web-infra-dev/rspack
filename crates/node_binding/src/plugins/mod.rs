mod interceptor;
use std::fmt;

use async_trait::async_trait;
pub use interceptor::RegisterJsTapKind;
pub use interceptor::RegisterJsTaps;
use napi::{Env, Result};
use rspack_core::{ApplyContext, CompilerOptions, PluginContext};
use rspack_hook::Hook as _;

use self::interceptor::*;

#[derive(Clone)]
pub struct JsHooksAdapterPlugin {
  non_skippable_registers: NonSkippableRegisters,
  register_compiler_this_compilation_taps: RegisterCompilerThisCompilationTaps,
  register_compiler_compilation_taps: RegisterCompilerCompilationTaps,
  register_compiler_make_taps: RegisterCompilerMakeTaps,
  register_compiler_finish_make_taps: RegisterCompilerFinishMakeTaps,
  register_compiler_should_emit_taps: RegisterCompilerShouldEmitTaps,
  register_compiler_emit_taps: RegisterCompilerEmitTaps,
  register_compiler_after_emit_taps: RegisterCompilerAfterEmitTaps,
  register_compiler_asset_emitted_taps: RegisterCompilerAssetEmittedTaps,
  register_compilation_build_module_taps: RegisterCompilationBuildModuleTaps,
  register_compilation_still_valid_module_taps: RegisterCompilationStillValidModuleTaps,
  register_compilation_succeed_module_taps: RegisterCompilationSucceedModuleTaps,
  register_compilation_execute_module_taps: RegisterCompilationExecuteModuleTaps,
  register_compilation_finish_modules_taps: RegisterCompilationFinishModulesTaps,
  register_compilation_optimize_modules_taps: RegisterCompilationOptimizeModulesTaps,
  register_compilation_after_optimize_modules_taps: RegisterCompilationAfterOptimizeModulesTaps,
  register_compilation_optimize_tree_taps: RegisterCompilationOptimizeTreeTaps,
  register_compilation_optimize_chunk_modules_taps: RegisterCompilationOptimizeChunkModulesTaps,
  register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps,
  register_compilation_chunk_asset_taps: RegisterCompilationChunkAssetTaps,
  register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps,
  register_compilation_after_process_assets_taps: RegisterCompilationAfterProcessAssetsTaps,
  register_compilation_after_seal_taps: RegisterCompilationAfterSealTaps,
  register_normal_module_factory_before_resolve_taps: RegisterNormalModuleFactoryBeforeResolveTaps,
  register_normal_module_factory_resolve_for_scheme_taps:
    RegisterNormalModuleFactoryResolveForSchemeTaps,
  register_normal_module_factory_after_resolve_taps: RegisterNormalModuleFactoryAfterResolveTaps,
  register_normal_module_factory_create_module_taps: RegisterNormalModuleFactoryCreateModuleTaps,
  register_context_module_factory_before_resolve_taps:
    RegisterContextModuleFactoryBeforeResolveTaps,
  register_context_module_factory_after_resolve_taps: RegisterContextModuleFactoryAfterResolveTaps,
}

impl fmt::Debug for JsHooksAdapterPlugin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "rspack_plugin_js_hooks_adapter")
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
      .finish_make
      .intercept(self.register_compiler_finish_make_taps.clone());
    ctx
      .context
      .compiler_hooks
      .should_emit
      .intercept(self.register_compiler_should_emit_taps.clone());
    ctx
      .context
      .compiler_hooks
      .emit
      .intercept(self.register_compiler_emit_taps.clone());
    ctx
      .context
      .compiler_hooks
      .after_emit
      .intercept(self.register_compiler_after_emit_taps.clone());
    ctx
      .context
      .compiler_hooks
      .asset_emitted
      .intercept(self.register_compiler_asset_emitted_taps.clone());
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
      .finish_modules
      .intercept(self.register_compilation_finish_modules_taps.clone());
    ctx
      .context
      .compilation_hooks
      .optimize_modules
      .intercept(self.register_compilation_optimize_modules_taps.clone());
    ctx
      .context
      .compilation_hooks
      .after_optimize_modules
      .intercept(
        self
          .register_compilation_after_optimize_modules_taps
          .clone(),
      );
    ctx
      .context
      .compilation_hooks
      .optimize_tree
      .intercept(self.register_compilation_optimize_tree_taps.clone());
    ctx
      .context
      .compilation_hooks
      .optimize_chunk_modules
      .intercept(
        self
          .register_compilation_optimize_chunk_modules_taps
          .clone(),
      );
    ctx
      .context
      .compilation_hooks
      .runtime_module
      .intercept(self.register_compilation_runtime_module_taps.clone());
    ctx
      .context
      .compilation_hooks
      .chunk_asset
      .intercept(self.register_compilation_chunk_asset_taps.clone());
    ctx
      .context
      .compilation_hooks
      .process_assets
      .intercept(self.register_compilation_process_assets_taps.clone());
    ctx
      .context
      .compilation_hooks
      .after_process_assets
      .intercept(self.register_compilation_after_process_assets_taps.clone());
    ctx
      .context
      .compilation_hooks
      .after_seal
      .intercept(self.register_compilation_after_seal_taps.clone());

    ctx
      .context
      .normal_module_factory_hooks
      .before_resolve
      .intercept(
        self
          .register_normal_module_factory_before_resolve_taps
          .clone(),
      );
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_for_scheme
      .intercept(
        self
          .register_normal_module_factory_resolve_for_scheme_taps
          .clone(),
      );
    ctx
      .context
      .normal_module_factory_hooks
      .after_resolve
      .intercept(
        self
          .register_normal_module_factory_after_resolve_taps
          .clone(),
      );
    ctx
      .context
      .normal_module_factory_hooks
      .create_module
      .intercept(
        self
          .register_normal_module_factory_create_module_taps
          .clone(),
      );
    ctx
      .context
      .context_module_factory_hooks
      .before_resolve
      .intercept(
        self
          .register_context_module_factory_before_resolve_taps
          .clone(),
      );
    ctx
      .context
      .context_module_factory_hooks
      .after_resolve
      .intercept(
        self
          .register_context_module_factory_after_resolve_taps
          .clone(),
      );
    Ok(())
  }
}

impl JsHooksAdapterPlugin {
  pub fn from_js_hooks(_env: Env, register_js_taps: RegisterJsTaps) -> Result<Self> {
    let non_skippable_registers = NonSkippableRegisters::default();
    Ok(JsHooksAdapterPlugin {
      register_compiler_this_compilation_taps: RegisterCompilerThisCompilationTaps::new(
        register_js_taps.register_compiler_this_compilation_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_compilation_taps: RegisterCompilerCompilationTaps::new(
        register_js_taps.register_compiler_compilation_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_make_taps: RegisterCompilerMakeTaps::new(
        register_js_taps.register_compiler_make_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_finish_make_taps: RegisterCompilerFinishMakeTaps::new(
        register_js_taps.register_compiler_finish_make_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_should_emit_taps: RegisterCompilerShouldEmitTaps::new(
        register_js_taps.register_compiler_should_emit_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_emit_taps: RegisterCompilerEmitTaps::new(
        register_js_taps.register_compiler_emit_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_after_emit_taps: RegisterCompilerAfterEmitTaps::new(
        register_js_taps.register_compiler_after_emit_taps,
        non_skippable_registers.clone(),
      ),
      register_compiler_asset_emitted_taps: RegisterCompilerAssetEmittedTaps::new(
        register_js_taps.register_compiler_asset_emitted_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_build_module_taps: RegisterCompilationBuildModuleTaps::new(
        register_js_taps.register_compilation_build_module_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_still_valid_module_taps: RegisterCompilationStillValidModuleTaps::new(
        register_js_taps.register_compilation_still_valid_module_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_succeed_module_taps: RegisterCompilationSucceedModuleTaps::new(
        register_js_taps.register_compilation_succeed_module_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_execute_module_taps: RegisterCompilationExecuteModuleTaps::new(
        register_js_taps.register_compilation_execute_module_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_finish_modules_taps: RegisterCompilationFinishModulesTaps::new(
        register_js_taps.register_compilation_finish_modules_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_optimize_modules_taps: RegisterCompilationOptimizeModulesTaps::new(
        register_js_taps.register_compilation_optimize_modules_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_after_optimize_modules_taps:
        RegisterCompilationAfterOptimizeModulesTaps::new(
          register_js_taps.register_compilation_after_optimize_modules_taps,
          non_skippable_registers.clone(),
        ),
      register_compilation_optimize_tree_taps: RegisterCompilationOptimizeTreeTaps::new(
        register_js_taps.register_compilation_optimize_tree_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_optimize_chunk_modules_taps:
        RegisterCompilationOptimizeChunkModulesTaps::new(
          register_js_taps.register_compilation_optimize_chunk_modules_taps,
          non_skippable_registers.clone(),
        ),
      register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps::new(
        register_js_taps.register_compilation_runtime_module_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_chunk_asset_taps: RegisterCompilationChunkAssetTaps::new(
        register_js_taps.register_compilation_chunk_asset_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps::new(
        register_js_taps.register_compilation_process_assets_taps,
        non_skippable_registers.clone(),
      ),
      register_compilation_after_process_assets_taps:
        RegisterCompilationAfterProcessAssetsTaps::new(
          register_js_taps.register_compilation_after_process_assets_taps,
          non_skippable_registers.clone(),
        ),
      register_compilation_after_seal_taps: RegisterCompilationAfterSealTaps::new(
        register_js_taps.register_compilation_after_seal_taps,
        non_skippable_registers.clone(),
      ),
      register_normal_module_factory_before_resolve_taps:
        RegisterNormalModuleFactoryBeforeResolveTaps::new(
          register_js_taps.register_normal_module_factory_before_resolve_taps,
          non_skippable_registers.clone(),
        ),
      register_normal_module_factory_resolve_for_scheme_taps:
        RegisterNormalModuleFactoryResolveForSchemeTaps::new(
          register_js_taps.register_normal_module_factory_resolve_for_scheme_taps,
          non_skippable_registers.clone(),
        ),
      register_normal_module_factory_after_resolve_taps:
        RegisterNormalModuleFactoryAfterResolveTaps::new(
          register_js_taps.register_normal_module_factory_after_resolve_taps,
          non_skippable_registers.clone(),
        ),
      register_normal_module_factory_create_module_taps:
        RegisterNormalModuleFactoryCreateModuleTaps::new(
          register_js_taps.register_normal_module_factory_create_module_taps,
          non_skippable_registers.clone(),
        ),
      register_context_module_factory_before_resolve_taps:
        RegisterContextModuleFactoryBeforeResolveTaps::new(
          register_js_taps.register_context_module_factory_before_resolve_taps,
          non_skippable_registers.clone(),
        ),
      register_context_module_factory_after_resolve_taps:
        RegisterContextModuleFactoryAfterResolveTaps::new(
          register_js_taps.register_context_module_factory_after_resolve_taps,
          non_skippable_registers.clone(),
        ),
      non_skippable_registers,
    })
  }

  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self
      .non_skippable_registers
      .set_non_skippable_registers(kinds);
  }
}
