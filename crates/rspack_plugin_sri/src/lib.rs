mod asset;
mod config;
mod html;
mod integrity;
mod runtime;
mod util;

use std::sync::LazyLock;

use asset::{detect_unresolved_integrity, handle_assets, update_hash};
use config::SRICompilationContext;
pub use config::{
  IntegrityCallbackData, IntegrityCallbackFn, IntegrityHtmlPlugin,
  SubresourceIntegrityPluginOptions,
};
use html::{alter_asset_tag_groups, before_asset_tag_generation};
pub use integrity::SubresourceIntegrityHashFunction;
use rspack_core::{
  ChunkLoading, ChunkLoadingType, Compilation, CompilationId, CompilationParams,
  CompilerThisCompilation, CrossOriginLoading, Plugin, PluginContext,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_runtime::RuntimePlugin;
use rspack_util::fx_hash::FxDashMap;
use runtime::{create_script, handle_runtime, link_preload};
use rustc_hash::FxHashMap as HashMap;

static COMPILATION_INTEGRITY_MAP: LazyLock<FxDashMap<CompilationId, HashMap<String, String>>> =
  LazyLock::new(Default::default);

static COMPILATION_CONTEXT_MAP: LazyLock<FxDashMap<CompilationId, SRICompilationContext>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug)]
pub struct SubresourceIntegrityPlugin {
  pub options: SubresourceIntegrityPluginOptions,
}

impl SubresourceIntegrityPlugin {
  pub fn new(options: SubresourceIntegrityPluginOptions) -> Self {
    Self::new_inner(options)
  }

  pub fn get_compilation_sri_context(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, SRICompilationContext> {
    COMPILATION_CONTEXT_MAP
      .get(&id)
      .expect("should have sri context")
  }

  pub fn set_compilation_sri_context(id: CompilationId, ctx: SRICompilationContext) {
    COMPILATION_CONTEXT_MAP.insert(id, ctx);
  }

  pub fn get_compilation_integrities(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, HashMap<String, String>> {
    if !COMPILATION_INTEGRITY_MAP.contains_key(&id) {
      COMPILATION_INTEGRITY_MAP.insert(id, Default::default());
    }
    COMPILATION_INTEGRITY_MAP
      .get(&id)
      .expect("should have compilation integrities")
  }

  pub fn get_compilation_integrities_mut(
    id: CompilationId,
  ) -> dashmap::mapref::one::RefMut<'static, CompilationId, HashMap<String, String>> {
    COMPILATION_INTEGRITY_MAP.entry(id).or_default()
  }
}

#[plugin_hook(CompilerThisCompilation for SubresourceIntegrityPlugin, stage = -10000)]
async fn warn_non_web(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.push_diagnostic(Diagnostic::warn(
    "SubresourceIntegrity".to_string(),
    "This plugin is not useful for non-web targets.".to_string(),
  ));
  Ok(())
}

#[plugin_hook(CompilerThisCompilation for SubresourceIntegrityPlugin, stage = -10000)]
async fn handle_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let ctx = SRICompilationContext {
    fs: compilation.output_filesystem.clone(),
    output_path: compilation.options.output.path.clone(),
    cross_origin_loading: compilation.options.output.cross_origin_loading.clone(),
  };
  SubresourceIntegrityPlugin::set_compilation_sri_context(compilation.id(), ctx);

  let mut real_content_hash_plugin_hooks =
    RealContentHashPlugin::get_compilation_hooks_mut(compilation.id());
  real_content_hash_plugin_hooks
    .update_hash
    .tap(update_hash::new(self));

  if matches!(
    compilation.options.output.cross_origin_loading,
    CrossOriginLoading::Disable
  ) {
    compilation.push_diagnostic(Diagnostic::error(
      "SubresourceIntegrity".to_string(),
      "rspack option output.crossOriginLoading not set, code splitting will not work!".to_string(),
    ));
  }

  let mut runtime_plugin_hooks = RuntimePlugin::get_compilation_hooks_mut(compilation.id());
  runtime_plugin_hooks
    .create_script
    .tap(create_script::new(self));
  runtime_plugin_hooks
    .link_preload
    .tap(link_preload::new(self));

  if matches!(self.options.html_plugin, IntegrityHtmlPlugin::NativePlugin) {
    let mut html_plugin_hooks = HtmlRspackPlugin::get_compilation_hooks_mut(compilation.id());
    html_plugin_hooks
      .before_asset_tag_generation
      .tap(before_asset_tag_generation::new(self));
    html_plugin_hooks
      .alter_asset_tag_groups
      .tap(alter_asset_tag_groups::new(self));
  }

  Ok(())
}

impl Plugin for SubresourceIntegrityPlugin {
  fn name(&self) -> &'static str {
    "rspack.SubresourceIntegrityPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    if let ChunkLoading::Enable(chunk_loading) = &options.output.chunk_loading {
      if matches!(
        chunk_loading,
        ChunkLoadingType::Require | ChunkLoadingType::AsyncNode
      ) {
        ctx
          .context
          .compiler_hooks
          .this_compilation
          .tap(warn_non_web::new(self));

        return Ok(());
      }
    }

    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(handle_assets::new(self));

    ctx
      .context
      .compilation_hooks
      .after_process_assets
      .tap(detect_unresolved_integrity::new(self));

    ctx
      .context
      .compiler_hooks
      .this_compilation
      .tap(handle_compilation::new(self));

    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(handle_runtime::new(self));
    Ok(())
  }
}
