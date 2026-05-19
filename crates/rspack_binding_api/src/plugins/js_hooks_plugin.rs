use std::fmt;

use async_trait::async_trait;
use napi::{Env, Result};
use rspack_core::{
  ApplyContext, Compilation, CompilationId, CompilationParams, CompilerCompilation,
  CompilerOptions, Plugin,
};
use rspack_hook::{Hook as _, plugin, plugin_hook};
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_javascript::JsPlugin;
use rspack_plugin_rsdoctor::RsdoctorPlugin;
use rspack_plugin_runtime::RuntimePlugin;

use super::interceptor::*;

#[plugin]
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
  register_compilation_before_module_ids_taps: RegisterCompilationBeforeModuleIdsTaps,
  register_compilation_additional_tree_runtime_requirements_taps:
    RegisterCompilationAdditionalTreeRuntimeRequirementsTaps,
  register_compilation_runtime_requirement_in_tree_taps:
    RegisterCompilationRuntimeRequirementInTreeTaps,
  register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps,
  register_compilation_chunk_hash_taps: RegisterCompilationChunkHashTaps,
  register_compilation_chunk_asset_taps: RegisterCompilationChunkAssetTaps,
  register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps,
  register_compilation_after_process_assets_taps: RegisterCompilationAfterProcessAssetsTaps,
  register_compilation_seal_taps: RegisterCompilationSealTaps,
  register_compilation_after_seal_taps: RegisterCompilationAfterSealTaps,
  register_normal_module_factory_before_resolve_taps: RegisterNormalModuleFactoryBeforeResolveTaps,
  register_normal_module_factory_factorize_taps: RegisterNormalModuleFactoryFactorizeTaps,
  register_normal_module_factory_resolve_taps: RegisterNormalModuleFactoryResolveTaps,
  register_normal_module_factory_resolve_for_scheme_taps:
    RegisterNormalModuleFactoryResolveForSchemeTaps,
  register_normal_module_factory_after_resolve_taps: RegisterNormalModuleFactoryAfterResolveTaps,
  register_normal_module_factory_create_module_taps: RegisterNormalModuleFactoryCreateModuleTaps,
  register_context_module_factory_before_resolve_taps:
    RegisterContextModuleFactoryBeforeResolveTaps,
  register_context_module_factory_after_resolve_taps: RegisterContextModuleFactoryAfterResolveTaps,
  register_javascript_modules_chunk_hash_taps: RegisterJavascriptModulesChunkHashTaps,
  register_html_plugin_before_asset_tag_generation_taps:
    RegisterHtmlPluginBeforeAssetTagGenerationTaps,
  register_html_plugin_alter_asset_tags_taps: RegisterHtmlPluginAlterAssetTagsTaps,
  register_html_plugin_alter_asset_tag_groups_taps: RegisterHtmlPluginAlterAssetTagGroupsTaps,
  register_html_plugin_after_template_execution_taps: RegisterHtmlPluginAfterTemplateExecutionTaps,
  register_html_plugin_before_emit_taps: RegisterHtmlPluginBeforeEmitTaps,
  register_html_plugin_after_emit_taps: RegisterHtmlPluginAfterEmitTaps,
  register_runtime_plugin_create_script_taps: RegisterRuntimePluginCreateScriptTaps,
  register_runtime_plugin_create_link_taps: RegisterRuntimePluginCreateLinkTaps,
  register_runtime_plugin_link_preload_taps: RegisterRuntimePluginLinkPreloadTaps,
  register_runtime_plugin_link_prefetch_taps: RegisterRuntimePluginLinkPrefetchTaps,
  register_rsdoctor_plugin_module_graph_taps: RegisterRsdoctorPluginModuleGraphTaps,
  register_rsdoctor_plugin_chunk_graph_taps: RegisterRsdoctorPluginChunkGraphTaps,
  register_rsdoctor_plugin_assets_taps: RegisterRsdoctorPluginAssetsTaps,
  register_rsdoctor_plugin_module_ids_taps: RegisterRsdoctorPluginModuleIdsTaps,
  register_rsdoctor_plugin_module_sources_taps: RegisterRsdoctorPluginModuleSourcesTaps,
}

impl fmt::Debug for JsHooksAdapterPlugin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "rspack_plugin_js_hooks_adapter")
  }
}

impl Plugin for JsHooksAdapterPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsHooksAdapterPlugin"
  }

  // #[tracing::instrument("js_hooks_adapter::apply", skip_all)]
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .intercept(self.register_compiler_this_compilation_taps.clone());
    ctx
      .compiler_hooks
      .compilation
      .intercept(self.register_compiler_compilation_taps.clone());
    ctx
      .compiler_hooks
      .make
      .intercept(self.register_compiler_make_taps.clone());
    ctx
      .compiler_hooks
      .finish_make
      .intercept(self.register_compiler_finish_make_taps.clone());
    ctx
      .compiler_hooks
      .should_emit
      .intercept(self.register_compiler_should_emit_taps.clone());
    ctx
      .compiler_hooks
      .emit
      .intercept(self.register_compiler_emit_taps.clone());
    ctx
      .compiler_hooks
      .after_emit
      .intercept(self.register_compiler_after_emit_taps.clone());
    ctx
      .compiler_hooks
      .asset_emitted
      .intercept(self.register_compiler_asset_emitted_taps.clone());
    ctx
      .compilation_hooks
      .build_module
      .intercept(self.register_compilation_build_module_taps.clone());
    ctx
      .compilation_hooks
      .still_valid_module
      .intercept(self.register_compilation_still_valid_module_taps.clone());
    ctx
      .compilation_hooks
      .succeed_module
      .intercept(self.register_compilation_succeed_module_taps.clone());
    ctx
      .compilation_hooks
      .execute_module
      .intercept(self.register_compilation_execute_module_taps.clone());
    ctx
      .compilation_hooks
      .finish_modules
      .intercept(self.register_compilation_finish_modules_taps.clone());
    ctx
      .compilation_hooks
      .optimize_modules
      .intercept(self.register_compilation_optimize_modules_taps.clone());
    ctx.compilation_hooks.after_optimize_modules.intercept(
      self
        .register_compilation_after_optimize_modules_taps
        .clone(),
    );
    ctx
      .compilation_hooks
      .optimize_tree
      .intercept(self.register_compilation_optimize_tree_taps.clone());
    ctx.compilation_hooks.optimize_chunk_modules.intercept(
      self
        .register_compilation_optimize_chunk_modules_taps
        .clone(),
    );
    ctx
      .compilation_hooks
      .before_module_ids
      .intercept(self.register_compilation_before_module_ids_taps.clone());
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .intercept(
        self
          .register_compilation_additional_tree_runtime_requirements_taps
          .clone(),
      );
    ctx.compilation_hooks.runtime_requirement_in_tree.intercept(
      self
        .register_compilation_runtime_requirement_in_tree_taps
        .clone(),
    );
    ctx
      .compilation_hooks
      .runtime_module
      .intercept(self.register_compilation_runtime_module_taps.clone());
    ctx
      .compilation_hooks
      .chunk_hash
      .intercept(self.register_compilation_chunk_hash_taps.clone());
    ctx
      .compilation_hooks
      .chunk_asset
      .intercept(self.register_compilation_chunk_asset_taps.clone());
    ctx
      .compilation_hooks
      .process_assets
      .intercept(self.register_compilation_process_assets_taps.clone());
    ctx
      .compilation_hooks
      .after_process_assets
      .intercept(self.register_compilation_after_process_assets_taps.clone());
    ctx
      .compilation_hooks
      .seal
      .intercept(self.register_compilation_seal_taps.clone());
    ctx
      .compilation_hooks
      .after_seal
      .intercept(self.register_compilation_after_seal_taps.clone());

    ctx.normal_module_factory_hooks.before_resolve.intercept(
      self
        .register_normal_module_factory_before_resolve_taps
        .clone(),
    );
    ctx
      .normal_module_factory_hooks
      .factorize
      .intercept(self.register_normal_module_factory_factorize_taps.clone());
    ctx
      .normal_module_factory_hooks
      .resolve
      .intercept(self.register_normal_module_factory_resolve_taps.clone());
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .intercept(
        self
          .register_normal_module_factory_resolve_for_scheme_taps
          .clone(),
      );
    ctx.normal_module_factory_hooks.after_resolve.intercept(
      self
        .register_normal_module_factory_after_resolve_taps
        .clone(),
    );
    ctx.normal_module_factory_hooks.create_module.intercept(
      self
        .register_normal_module_factory_create_module_taps
        .clone(),
    );
    ctx.context_module_factory_hooks.before_resolve.intercept(
      self
        .register_context_module_factory_before_resolve_taps
        .clone(),
    );
    ctx.context_module_factory_hooks.after_resolve.intercept(
      self
        .register_context_module_factory_after_resolve_taps
        .clone(),
    );

    ctx
      .compiler_hooks
      .compilation
      .tap(js_hooks_adapter_compilation::new(self));

    ctx
      .compiler_hooks
      .compilation
      .tap(html_hooks_adapter_compilation::new(self));

    ctx
      .compiler_hooks
      .compilation
      .tap(runtime_hooks_adapter_compilation::new(self));

    ctx
      .compiler_hooks
      .compilation
      .tap(rsdoctor_hooks_adapter_compilation::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: CompilationId) {
    self.register_compiler_this_compilation_taps.clear_cache();
    self.register_compiler_compilation_taps.clear_cache();
    self.register_compiler_make_taps.clear_cache();
    self.register_compiler_finish_make_taps.clear_cache();
    self.register_compiler_should_emit_taps.clear_cache();
    self.register_compiler_emit_taps.clear_cache();
    self.register_compiler_after_emit_taps.clear_cache();
    self.register_compiler_asset_emitted_taps.clear_cache();
    self.register_compilation_build_module_taps.clear_cache();
    self
      .register_compilation_still_valid_module_taps
      .clear_cache();
    self.register_compilation_succeed_module_taps.clear_cache();
    self.register_compilation_execute_module_taps.clear_cache();
    self.register_compilation_finish_modules_taps.clear_cache();
    self
      .register_compilation_optimize_modules_taps
      .clear_cache();
    self
      .register_compilation_after_optimize_modules_taps
      .clear_cache();
    self.register_compilation_optimize_tree_taps.clear_cache();
    self
      .register_compilation_optimize_chunk_modules_taps
      .clear_cache();
    self
      .register_compilation_before_module_ids_taps
      .clear_cache();
    self
      .register_compilation_additional_tree_runtime_requirements_taps
      .clear_cache();
    self
      .register_compilation_runtime_requirement_in_tree_taps
      .clear_cache();
    self.register_compilation_runtime_module_taps.clear_cache();
    self.register_compilation_chunk_hash_taps.clear_cache();
    self.register_compilation_chunk_asset_taps.clear_cache();
    self.register_compilation_process_assets_taps.clear_cache();
    self
      .register_compilation_after_process_assets_taps
      .clear_cache();
    self.register_compilation_seal_taps.clear_cache();
    self.register_compilation_after_seal_taps.clear_cache();
    self
      .register_normal_module_factory_before_resolve_taps
      .clear_cache();
    self
      .register_normal_module_factory_factorize_taps
      .clear_cache();
    self
      .register_normal_module_factory_resolve_taps
      .clear_cache();
    self
      .register_normal_module_factory_resolve_for_scheme_taps
      .clear_cache();
    self
      .register_normal_module_factory_after_resolve_taps
      .clear_cache();
    self
      .register_normal_module_factory_create_module_taps
      .clear_cache();
    self
      .register_context_module_factory_before_resolve_taps
      .clear_cache();
    self
      .register_context_module_factory_after_resolve_taps
      .clear_cache();
    self
      .register_javascript_modules_chunk_hash_taps
      .clear_cache();
    self
      .register_html_plugin_before_asset_tag_generation_taps
      .clear_cache();
    self
      .register_html_plugin_alter_asset_tags_taps
      .clear_cache();
    self
      .register_html_plugin_alter_asset_tag_groups_taps
      .clear_cache();
    self
      .register_html_plugin_after_template_execution_taps
      .clear_cache();
    self.register_html_plugin_before_emit_taps.clear_cache();
    self.register_html_plugin_after_emit_taps.clear_cache();
    self
      .register_runtime_plugin_create_script_taps
      .clear_cache();
    self.register_runtime_plugin_link_preload_taps.clear_cache();
    self
      .register_runtime_plugin_link_prefetch_taps
      .clear_cache();
    self
      .register_rsdoctor_plugin_module_graph_taps
      .clear_cache();
    self.register_rsdoctor_plugin_chunk_graph_taps.clear_cache();
    self.register_rsdoctor_plugin_assets_taps.clear_cache();
    self.register_rsdoctor_plugin_module_ids_taps.clear_cache();
    self
      .register_rsdoctor_plugin_module_sources_taps
      .clear_cache();
  }
}

#[plugin_hook(CompilerCompilation for JsHooksAdapterPlugin)]
async fn js_hooks_adapter_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> rspack_error::Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks
    .chunk_hash
    .intercept(self.register_javascript_modules_chunk_hash_taps.clone());

  Ok(())
}

#[plugin_hook(CompilerCompilation for JsHooksAdapterPlugin)]
async fn html_hooks_adapter_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> rspack_error::Result<()> {
  let hooks = HtmlRspackPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.borrow_mut();
  hooks.before_asset_tag_generation.intercept(
    self
      .register_html_plugin_before_asset_tag_generation_taps
      .clone(),
  );
  hooks
    .alter_asset_tags
    .intercept(self.register_html_plugin_alter_asset_tags_taps.clone());
  hooks.alter_asset_tag_groups.intercept(
    self
      .register_html_plugin_alter_asset_tag_groups_taps
      .clone(),
  );
  hooks.after_template_execution.intercept(
    self
      .register_html_plugin_after_template_execution_taps
      .clone(),
  );
  hooks
    .before_emit
    .intercept(self.register_html_plugin_before_emit_taps.clone());
  hooks
    .after_emit
    .intercept(self.register_html_plugin_after_emit_taps.clone());

  Ok(())
}

#[plugin_hook(CompilerCompilation for JsHooksAdapterPlugin)]
async fn runtime_hooks_adapter_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> rspack_error::Result<()> {
  let hooks = RuntimePlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.borrow_mut();
  hooks
    .create_script
    .intercept(self.register_runtime_plugin_create_script_taps.clone());
  hooks
    .create_link
    .intercept(self.register_runtime_plugin_create_link_taps.clone());
  hooks
    .link_preload
    .intercept(self.register_runtime_plugin_link_preload_taps.clone());
  hooks
    .link_prefetch
    .intercept(self.register_runtime_plugin_link_prefetch_taps.clone());
  Ok(())
}

#[plugin_hook(CompilerCompilation for JsHooksAdapterPlugin)]
async fn rsdoctor_hooks_adapter_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> rspack_error::Result<()> {
  let hooks = RsdoctorPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.borrow_mut();
  hooks
    .module_graph
    .intercept(self.register_rsdoctor_plugin_module_graph_taps.clone());
  hooks
    .chunk_graph
    .intercept(self.register_rsdoctor_plugin_chunk_graph_taps.clone());
  hooks
    .assets
    .intercept(self.register_rsdoctor_plugin_assets_taps.clone());
  hooks
    .module_ids
    .intercept(self.register_rsdoctor_plugin_module_ids_taps.clone());
  hooks
    .module_sources
    .intercept(self.register_rsdoctor_plugin_module_sources_taps.clone());

  Ok(())
}

impl JsHooksAdapterPlugin {
  pub fn from_js_hooks(_env: Env, register_js_taps: RegisterJsTaps) -> Result<Self> {
    let non_skippable_registers = NonSkippableRegisters::default();
    Ok(JsHooksAdapterPlugin {
      inner: JsHooksAdapterPluginInner {
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
        register_compilation_before_module_ids_taps: RegisterCompilationBeforeModuleIdsTaps::new(
          register_js_taps.register_compilation_before_module_ids_taps,
          non_skippable_registers.clone(),
        ),
        register_compilation_additional_tree_runtime_requirements_taps:
          RegisterCompilationAdditionalTreeRuntimeRequirementsTaps::new(
            register_js_taps.register_compilation_additional_tree_runtime_requirements_taps,
            non_skippable_registers.clone(),
          ),
        register_compilation_runtime_requirement_in_tree_taps:
          RegisterCompilationRuntimeRequirementInTreeTaps::new(
            register_js_taps.register_compilation_runtime_requirement_in_tree_taps,
            non_skippable_registers.clone(),
          ),
        register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps::new(
          register_js_taps.register_compilation_runtime_module_taps,
          non_skippable_registers.clone(),
        ),
        register_compilation_chunk_hash_taps: RegisterCompilationChunkHashTaps::new(
          register_js_taps.register_compilation_chunk_hash_taps,
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
        register_compilation_seal_taps: RegisterCompilationSealTaps::new(
          register_js_taps.register_compilation_seal_taps,
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
        register_normal_module_factory_factorize_taps:
          RegisterNormalModuleFactoryFactorizeTaps::new(
            register_js_taps.register_normal_module_factory_factorize_taps,
            non_skippable_registers.clone(),
          ),
        register_normal_module_factory_resolve_taps: RegisterNormalModuleFactoryResolveTaps::new(
          register_js_taps.register_normal_module_factory_resolve_taps,
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
        register_javascript_modules_chunk_hash_taps: RegisterJavascriptModulesChunkHashTaps::new(
          register_js_taps.register_javascript_modules_chunk_hash_taps,
          non_skippable_registers.clone(),
        ),
        register_html_plugin_before_asset_tag_generation_taps:
          RegisterHtmlPluginBeforeAssetTagGenerationTaps::new(
            register_js_taps.register_html_plugin_before_asset_tag_generation_taps,
            non_skippable_registers.clone(),
          ),
        register_html_plugin_alter_asset_tags_taps: RegisterHtmlPluginAlterAssetTagsTaps::new(
          register_js_taps.register_html_plugin_alter_asset_tags_taps,
          non_skippable_registers.clone(),
        ),
        register_html_plugin_alter_asset_tag_groups_taps:
          RegisterHtmlPluginAlterAssetTagGroupsTaps::new(
            register_js_taps.register_html_plugin_alter_asset_tag_groups_taps,
            non_skippable_registers.clone(),
          ),
        register_html_plugin_after_template_execution_taps:
          RegisterHtmlPluginAfterTemplateExecutionTaps::new(
            register_js_taps.register_html_plugin_after_template_execution_taps,
            non_skippable_registers.clone(),
          ),
        register_html_plugin_before_emit_taps: RegisterHtmlPluginBeforeEmitTaps::new(
          register_js_taps.register_html_plugin_before_emit_taps,
          non_skippable_registers.clone(),
        ),
        register_html_plugin_after_emit_taps: RegisterHtmlPluginAfterEmitTaps::new(
          register_js_taps.register_html_plugin_after_emit_taps,
          non_skippable_registers.clone(),
        ),
        register_runtime_plugin_create_script_taps: RegisterRuntimePluginCreateScriptTaps::new(
          register_js_taps.register_runtime_plugin_create_script_taps,
          non_skippable_registers.clone(),
        ),
        register_runtime_plugin_create_link_taps: RegisterRuntimePluginCreateLinkTaps::new(
          register_js_taps.register_runtime_plugin_create_link_taps,
          non_skippable_registers.clone(),
        ),
        register_runtime_plugin_link_preload_taps: RegisterRuntimePluginLinkPreloadTaps::new(
          register_js_taps.register_runtime_plugin_link_preload_taps,
          non_skippable_registers.clone(),
        ),
        register_runtime_plugin_link_prefetch_taps: RegisterRuntimePluginLinkPrefetchTaps::new(
          register_js_taps.register_runtime_plugin_link_prefetch_taps,
          non_skippable_registers.clone(),
        ),
        register_rsdoctor_plugin_module_graph_taps: RegisterRsdoctorPluginModuleGraphTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_graph_taps,
          non_skippable_registers.clone(),
        ),
        register_rsdoctor_plugin_chunk_graph_taps: RegisterRsdoctorPluginChunkGraphTaps::new(
          register_js_taps.register_rsdoctor_plugin_chunk_graph_taps,
          non_skippable_registers.clone(),
        ),
        register_rsdoctor_plugin_assets_taps: RegisterRsdoctorPluginAssetsTaps::new(
          register_js_taps.register_rsdoctor_plugin_assets_taps,
          non_skippable_registers.clone(),
        ),
        register_rsdoctor_plugin_module_ids_taps: RegisterRsdoctorPluginModuleIdsTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_ids_taps,
          non_skippable_registers.clone(),
        ),
        register_rsdoctor_plugin_module_sources_taps: RegisterRsdoctorPluginModuleSourcesTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_sources_taps,
          non_skippable_registers.clone(),
        ),
        non_skippable_registers,
      }
      .into(),
    })
  }

  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self
      .non_skippable_registers
      .set_non_skippable_registers(kinds);
  }
}
