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
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_rsdoctor::RsdoctorPlugin;
use rspack_plugin_runtime::RuntimePlugin;

use super::interceptor::*;

#[plugin]
#[derive(Clone)]
pub struct JsHooksAdapterPlugin {
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
  register_real_content_hash_plugin_update_hash_taps: RegisterRealContentHashPluginUpdateHashTaps,
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
    ctx.compiler_hooks.this_compilation.load_js_tap_register(
      self
        .register_compiler_this_compilation_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.compilation.load_js_tap_register(
      self
        .register_compiler_compilation_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.make.load_js_tap_register(
      self
        .register_compiler_make_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.finish_make.load_js_tap_register(
      self
        .register_compiler_finish_make_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.should_emit.load_js_tap_register(
      self
        .register_compiler_should_emit_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.emit.load_js_tap_register(
      self
        .register_compiler_emit_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.after_emit.load_js_tap_register(
      self
        .register_compiler_after_emit_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compiler_hooks.asset_emitted.load_js_tap_register(
      self
        .register_compiler_asset_emitted_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.build_module.load_js_tap_register(
      self
        .register_compilation_build_module_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx
      .compilation_hooks
      .still_valid_module
      .load_js_tap_register(
        self
          .register_compilation_still_valid_module_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx.compilation_hooks.succeed_module.load_js_tap_register(
      self
        .register_compilation_succeed_module_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.execute_module.load_js_tap_register(
      self
        .register_compilation_execute_module_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.finish_modules.load_js_tap_register(
      self
        .register_compilation_finish_modules_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx
      .compilation_hooks
      .optimize_modules
      .load_js_tap_register(
        self
          .register_compilation_optimize_modules_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .compilation_hooks
      .after_optimize_modules
      .load_js_tap_register(
        self
          .register_compilation_after_optimize_modules_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx.compilation_hooks.optimize_tree.load_js_tap_register(
      self
        .register_compilation_optimize_tree_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .load_js_tap_register(
        self
          .register_compilation_optimize_chunk_modules_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .compilation_hooks
      .before_module_ids
      .load_js_tap_register(
        self
          .register_compilation_before_module_ids_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .load_js_tap_register(
        self
          .register_compilation_additional_tree_runtime_requirements_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .load_js_tap_register(
        self
          .register_compilation_runtime_requirement_in_tree_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx.compilation_hooks.runtime_module.load_js_tap_register(
      self
        .register_compilation_runtime_module_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.chunk_hash.load_js_tap_register(
      self
        .register_compilation_chunk_hash_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.chunk_asset.load_js_tap_register(
      self
        .register_compilation_chunk_asset_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.process_assets.load_js_tap_register(
      self
        .register_compilation_process_assets_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx
      .compilation_hooks
      .after_process_assets
      .load_js_tap_register(
        self
          .register_compilation_after_process_assets_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx.compilation_hooks.seal.load_js_tap_register(
      self
        .register_compilation_seal_taps
        .clone()
        .into_js_tap_register(),
    )?;
    ctx.compilation_hooks.after_seal.load_js_tap_register(
      self
        .register_compilation_after_seal_taps
        .clone()
        .into_js_tap_register(),
    )?;

    ctx
      .normal_module_factory_hooks
      .before_resolve
      .load_js_tap_register(
        self
          .register_normal_module_factory_before_resolve_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .normal_module_factory_hooks
      .factorize
      .load_js_tap_register(
        self
          .register_normal_module_factory_factorize_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .normal_module_factory_hooks
      .resolve
      .load_js_tap_register(
        self
          .register_normal_module_factory_resolve_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .load_js_tap_register(
        self
          .register_normal_module_factory_resolve_for_scheme_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .normal_module_factory_hooks
      .after_resolve
      .load_js_tap_register(
        self
          .register_normal_module_factory_after_resolve_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .normal_module_factory_hooks
      .create_module
      .load_js_tap_register(
        self
          .register_normal_module_factory_create_module_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .context_module_factory_hooks
      .before_resolve
      .load_js_tap_register(
        self
          .register_context_module_factory_before_resolve_taps
          .clone()
          .into_js_tap_register(),
      )?;
    ctx
      .context_module_factory_hooks
      .after_resolve
      .load_js_tap_register(
        self
          .register_context_module_factory_after_resolve_taps
          .clone()
          .into_js_tap_register(),
      )?;

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
      .tap(real_content_hash_hooks_adapter_compilation::new(self));

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
    self.register_runtime_plugin_create_link_taps.clear_cache();
    self.register_runtime_plugin_link_preload_taps.clear_cache();
    self
      .register_runtime_plugin_link_prefetch_taps
      .clear_cache();
    self
      .register_real_content_hash_plugin_update_hash_taps
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
  hooks.chunk_hash.load_js_tap_register(
    self
      .register_javascript_modules_chunk_hash_taps
      .clone()
      .into_js_tap_register(),
  )?;

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
  hooks.before_asset_tag_generation.load_js_tap_register(
    self
      .register_html_plugin_before_asset_tag_generation_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.alter_asset_tags.load_js_tap_register(
    self
      .register_html_plugin_alter_asset_tags_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.alter_asset_tag_groups.load_js_tap_register(
    self
      .register_html_plugin_alter_asset_tag_groups_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.after_template_execution.load_js_tap_register(
    self
      .register_html_plugin_after_template_execution_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.before_emit.load_js_tap_register(
    self
      .register_html_plugin_before_emit_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.after_emit.load_js_tap_register(
    self
      .register_html_plugin_after_emit_taps
      .clone()
      .into_js_tap_register(),
  )?;

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
  hooks.create_script.load_js_tap_register(
    self
      .register_runtime_plugin_create_script_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.create_link.load_js_tap_register(
    self
      .register_runtime_plugin_create_link_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.link_preload.load_js_tap_register(
    self
      .register_runtime_plugin_link_preload_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.link_prefetch.load_js_tap_register(
    self
      .register_runtime_plugin_link_prefetch_taps
      .clone()
      .into_js_tap_register(),
  )?;
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
  hooks.module_graph.load_js_tap_register(
    self
      .register_rsdoctor_plugin_module_graph_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.chunk_graph.load_js_tap_register(
    self
      .register_rsdoctor_plugin_chunk_graph_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.assets.load_js_tap_register(
    self
      .register_rsdoctor_plugin_assets_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.module_ids.load_js_tap_register(
    self
      .register_rsdoctor_plugin_module_ids_taps
      .clone()
      .into_js_tap_register(),
  )?;
  hooks.module_sources.load_js_tap_register(
    self
      .register_rsdoctor_plugin_module_sources_taps
      .clone()
      .into_js_tap_register(),
  )?;

  Ok(())
}

#[plugin_hook(CompilerCompilation for JsHooksAdapterPlugin)]
async fn real_content_hash_hooks_adapter_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> rspack_error::Result<()> {
  let hooks = RealContentHashPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.borrow_mut();
  hooks.update_hash.load_js_tap_register(
    self
      .register_real_content_hash_plugin_update_hash_taps
      .clone()
      .into_js_tap_register(),
  )?;
  Ok(())
}

impl JsHooksAdapterPlugin {
  /// The `_env` parameter ensures this function is called on the JS main thread.
  pub fn from_js_hooks(_env: &Env, register_js_taps: RegisterJsTaps) -> Result<Self> {
    Ok(JsHooksAdapterPlugin {
      inner: JsHooksAdapterPluginInner {
        register_compiler_this_compilation_taps: RegisterCompilerThisCompilationTaps::new(
          register_js_taps.register_compiler_this_compilation_taps,
        ),
        register_compiler_compilation_taps: RegisterCompilerCompilationTaps::new(
          register_js_taps.register_compiler_compilation_taps,
        ),
        register_compiler_make_taps: RegisterCompilerMakeTaps::new(
          register_js_taps.register_compiler_make_taps,
        ),
        register_compiler_finish_make_taps: RegisterCompilerFinishMakeTaps::new(
          register_js_taps.register_compiler_finish_make_taps,
        ),
        register_compiler_should_emit_taps: RegisterCompilerShouldEmitTaps::new(
          register_js_taps.register_compiler_should_emit_taps,
        ),
        register_compiler_emit_taps: RegisterCompilerEmitTaps::new(
          register_js_taps.register_compiler_emit_taps,
        ),
        register_compiler_after_emit_taps: RegisterCompilerAfterEmitTaps::new(
          register_js_taps.register_compiler_after_emit_taps,
        ),
        register_compiler_asset_emitted_taps: RegisterCompilerAssetEmittedTaps::new(
          register_js_taps.register_compiler_asset_emitted_taps,
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
        register_compilation_finish_modules_taps: RegisterCompilationFinishModulesTaps::new(
          register_js_taps.register_compilation_finish_modules_taps,
        ),
        register_compilation_optimize_modules_taps: RegisterCompilationOptimizeModulesTaps::new(
          register_js_taps.register_compilation_optimize_modules_taps,
        ),
        register_compilation_after_optimize_modules_taps:
          RegisterCompilationAfterOptimizeModulesTaps::new(
            register_js_taps.register_compilation_after_optimize_modules_taps,
          ),
        register_compilation_optimize_tree_taps: RegisterCompilationOptimizeTreeTaps::new(
          register_js_taps.register_compilation_optimize_tree_taps,
        ),
        register_compilation_optimize_chunk_modules_taps:
          RegisterCompilationOptimizeChunkModulesTaps::new(
            register_js_taps.register_compilation_optimize_chunk_modules_taps,
          ),
        register_compilation_before_module_ids_taps: RegisterCompilationBeforeModuleIdsTaps::new(
          register_js_taps.register_compilation_before_module_ids_taps,
        ),
        register_compilation_additional_tree_runtime_requirements_taps:
          RegisterCompilationAdditionalTreeRuntimeRequirementsTaps::new(
            register_js_taps.register_compilation_additional_tree_runtime_requirements_taps,
          ),
        register_compilation_runtime_requirement_in_tree_taps:
          RegisterCompilationRuntimeRequirementInTreeTaps::new(
            register_js_taps.register_compilation_runtime_requirement_in_tree_taps,
          ),
        register_compilation_runtime_module_taps: RegisterCompilationRuntimeModuleTaps::new(
          register_js_taps.register_compilation_runtime_module_taps,
        ),
        register_compilation_chunk_hash_taps: RegisterCompilationChunkHashTaps::new(
          register_js_taps.register_compilation_chunk_hash_taps,
        ),
        register_compilation_chunk_asset_taps: RegisterCompilationChunkAssetTaps::new(
          register_js_taps.register_compilation_chunk_asset_taps,
        ),
        register_compilation_process_assets_taps: RegisterCompilationProcessAssetsTaps::new(
          register_js_taps.register_compilation_process_assets_taps,
        ),
        register_compilation_after_process_assets_taps:
          RegisterCompilationAfterProcessAssetsTaps::new(
            register_js_taps.register_compilation_after_process_assets_taps,
          ),
        register_compilation_seal_taps: RegisterCompilationSealTaps::new(
          register_js_taps.register_compilation_seal_taps,
        ),
        register_compilation_after_seal_taps: RegisterCompilationAfterSealTaps::new(
          register_js_taps.register_compilation_after_seal_taps,
        ),
        register_normal_module_factory_before_resolve_taps:
          RegisterNormalModuleFactoryBeforeResolveTaps::new(
            register_js_taps.register_normal_module_factory_before_resolve_taps,
          ),
        register_normal_module_factory_factorize_taps:
          RegisterNormalModuleFactoryFactorizeTaps::new(
            register_js_taps.register_normal_module_factory_factorize_taps,
          ),
        register_normal_module_factory_resolve_taps: RegisterNormalModuleFactoryResolveTaps::new(
          register_js_taps.register_normal_module_factory_resolve_taps,
        ),
        register_normal_module_factory_resolve_for_scheme_taps:
          RegisterNormalModuleFactoryResolveForSchemeTaps::new(
            register_js_taps.register_normal_module_factory_resolve_for_scheme_taps,
          ),
        register_normal_module_factory_after_resolve_taps:
          RegisterNormalModuleFactoryAfterResolveTaps::new(
            register_js_taps.register_normal_module_factory_after_resolve_taps,
          ),
        register_normal_module_factory_create_module_taps:
          RegisterNormalModuleFactoryCreateModuleTaps::new(
            register_js_taps.register_normal_module_factory_create_module_taps,
          ),
        register_context_module_factory_before_resolve_taps:
          RegisterContextModuleFactoryBeforeResolveTaps::new(
            register_js_taps.register_context_module_factory_before_resolve_taps,
          ),
        register_context_module_factory_after_resolve_taps:
          RegisterContextModuleFactoryAfterResolveTaps::new(
            register_js_taps.register_context_module_factory_after_resolve_taps,
          ),
        register_javascript_modules_chunk_hash_taps: RegisterJavascriptModulesChunkHashTaps::new(
          register_js_taps.register_javascript_modules_chunk_hash_taps,
        ),
        register_html_plugin_before_asset_tag_generation_taps:
          RegisterHtmlPluginBeforeAssetTagGenerationTaps::new(
            register_js_taps.register_html_plugin_before_asset_tag_generation_taps,
          ),
        register_html_plugin_alter_asset_tags_taps: RegisterHtmlPluginAlterAssetTagsTaps::new(
          register_js_taps.register_html_plugin_alter_asset_tags_taps,
        ),
        register_html_plugin_alter_asset_tag_groups_taps:
          RegisterHtmlPluginAlterAssetTagGroupsTaps::new(
            register_js_taps.register_html_plugin_alter_asset_tag_groups_taps,
          ),
        register_html_plugin_after_template_execution_taps:
          RegisterHtmlPluginAfterTemplateExecutionTaps::new(
            register_js_taps.register_html_plugin_after_template_execution_taps,
          ),
        register_html_plugin_before_emit_taps: RegisterHtmlPluginBeforeEmitTaps::new(
          register_js_taps.register_html_plugin_before_emit_taps,
        ),
        register_html_plugin_after_emit_taps: RegisterHtmlPluginAfterEmitTaps::new(
          register_js_taps.register_html_plugin_after_emit_taps,
        ),
        register_runtime_plugin_create_script_taps: RegisterRuntimePluginCreateScriptTaps::new(
          register_js_taps.register_runtime_plugin_create_script_taps,
        ),
        register_runtime_plugin_create_link_taps: RegisterRuntimePluginCreateLinkTaps::new(
          register_js_taps.register_runtime_plugin_create_link_taps,
        ),
        register_runtime_plugin_link_preload_taps: RegisterRuntimePluginLinkPreloadTaps::new(
          register_js_taps.register_runtime_plugin_link_preload_taps,
        ),
        register_runtime_plugin_link_prefetch_taps: RegisterRuntimePluginLinkPrefetchTaps::new(
          register_js_taps.register_runtime_plugin_link_prefetch_taps,
        ),
        register_real_content_hash_plugin_update_hash_taps:
          RegisterRealContentHashPluginUpdateHashTaps::new(
            register_js_taps.register_real_content_hash_plugin_update_hash_taps,
          ),
        register_rsdoctor_plugin_module_graph_taps: RegisterRsdoctorPluginModuleGraphTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_graph_taps,
        ),
        register_rsdoctor_plugin_chunk_graph_taps: RegisterRsdoctorPluginChunkGraphTaps::new(
          register_js_taps.register_rsdoctor_plugin_chunk_graph_taps,
        ),
        register_rsdoctor_plugin_assets_taps: RegisterRsdoctorPluginAssetsTaps::new(
          register_js_taps.register_rsdoctor_plugin_assets_taps,
        ),
        register_rsdoctor_plugin_module_ids_taps: RegisterRsdoctorPluginModuleIdsTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_ids_taps,
        ),
        register_rsdoctor_plugin_module_sources_taps: RegisterRsdoctorPluginModuleSourcesTaps::new(
          register_js_taps.register_rsdoctor_plugin_module_sources_taps,
        ),
      }
      .into(),
    })
  }

  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    for kind in RegisterJsTapKind::ALL {
      self.set_register_js_tap_count(*kind, usize::from(kinds.contains(kind)));
    }
  }

  pub fn set_register_js_tap_count(&self, kind: RegisterJsTapKind, tap_count: usize) {
    match kind {
      RegisterJsTapKind::CompilerThisCompilation => self
        .register_compiler_this_compilation_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilerCompilation => self
        .register_compiler_compilation_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilerMake => self.register_compiler_make_taps.set_tap_count(tap_count),
      RegisterJsTapKind::CompilerFinishMake => self
        .register_compiler_finish_make_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilerShouldEmit => self
        .register_compiler_should_emit_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilerEmit => self.register_compiler_emit_taps.set_tap_count(tap_count),
      RegisterJsTapKind::CompilerAfterEmit => self
        .register_compiler_after_emit_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilerAssetEmitted => self
        .register_compiler_asset_emitted_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationBuildModule => self
        .register_compilation_build_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationStillValidModule => self
        .register_compilation_still_valid_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationSucceedModule => self
        .register_compilation_succeed_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationExecuteModule => self
        .register_compilation_execute_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationFinishModules => self
        .register_compilation_finish_modules_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationOptimizeModules => self
        .register_compilation_optimize_modules_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationAfterOptimizeModules => self
        .register_compilation_after_optimize_modules_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationOptimizeTree => self
        .register_compilation_optimize_tree_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationOptimizeChunkModules => self
        .register_compilation_optimize_chunk_modules_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationBeforeModuleIds => self
        .register_compilation_before_module_ids_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationAdditionalTreeRuntimeRequirements => self
        .register_compilation_additional_tree_runtime_requirements_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationRuntimeRequirementInTree => self
        .register_compilation_runtime_requirement_in_tree_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationRuntimeModule => self
        .register_compilation_runtime_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationChunkHash => self
        .register_compilation_chunk_hash_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationChunkAsset => self
        .register_compilation_chunk_asset_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationProcessAssets => self
        .register_compilation_process_assets_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationAfterProcessAssets => self
        .register_compilation_after_process_assets_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::CompilationSeal => {
        self.register_compilation_seal_taps.set_tap_count(tap_count)
      }
      RegisterJsTapKind::CompilationAfterSeal => self
        .register_compilation_after_seal_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryBeforeResolve => self
        .register_normal_module_factory_before_resolve_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryFactorize => self
        .register_normal_module_factory_factorize_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryResolve => self
        .register_normal_module_factory_resolve_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryAfterResolve => self
        .register_normal_module_factory_after_resolve_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryCreateModule => self
        .register_normal_module_factory_create_module_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::NormalModuleFactoryResolveForScheme => self
        .register_normal_module_factory_resolve_for_scheme_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::ContextModuleFactoryBeforeResolve => self
        .register_context_module_factory_before_resolve_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::ContextModuleFactoryAfterResolve => self
        .register_context_module_factory_after_resolve_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::JavascriptModulesChunkHash => self
        .register_javascript_modules_chunk_hash_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginBeforeAssetTagGeneration => self
        .register_html_plugin_before_asset_tag_generation_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginAlterAssetTags => self
        .register_html_plugin_alter_asset_tags_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginAlterAssetTagGroups => self
        .register_html_plugin_alter_asset_tag_groups_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginAfterTemplateExecution => self
        .register_html_plugin_after_template_execution_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginBeforeEmit => self
        .register_html_plugin_before_emit_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::HtmlPluginAfterEmit => self
        .register_html_plugin_after_emit_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RuntimePluginCreateScript => self
        .register_runtime_plugin_create_script_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RuntimePluginCreateLink => self
        .register_runtime_plugin_create_link_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RuntimePluginLinkPreload => self
        .register_runtime_plugin_link_preload_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RuntimePluginLinkPrefetch => self
        .register_runtime_plugin_link_prefetch_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RealContentHashPluginUpdateHash => self
        .register_real_content_hash_plugin_update_hash_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RsdoctorPluginModuleGraph => self
        .register_rsdoctor_plugin_module_graph_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RsdoctorPluginChunkGraph => self
        .register_rsdoctor_plugin_chunk_graph_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RsdoctorPluginModuleIds => self
        .register_rsdoctor_plugin_module_ids_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RsdoctorPluginModuleSources => self
        .register_rsdoctor_plugin_module_sources_taps
        .set_tap_count(tap_count),
      RegisterJsTapKind::RsdoctorPluginAssets => self
        .register_rsdoctor_plugin_assets_taps
        .set_tap_count(tap_count),
    }
  }
}
