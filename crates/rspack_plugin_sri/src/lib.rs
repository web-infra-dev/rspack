mod asset;
mod config;
mod html;
mod integrity;
mod runtime;
mod util;

use std::sync::{Arc, LazyLock};

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
  CompilerThisCompilation, CrossOriginLoading, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_runtime::RuntimePlugin;
#[cfg(allocative)]
use rspack_util::allocative;
use rspack_util::fx_hash::FxDashMap;
use runtime::{create_link, create_script, handle_runtime, link_preload};
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::RwLock;

type CompilationIntegrityMap =
  LazyLock<FxDashMap<CompilationId, Arc<RwLock<HashMap<String, String>>>>>;

#[cfg_attr(allocative, allocative::root)]
static COMPILATION_INTEGRITY_MAP: CompilationIntegrityMap = LazyLock::new(Default::default);
#[cfg_attr(allocative, allocative::root)]
static COMPILATION_CONTEXT_MAP: LazyLock<FxDashMap<CompilationId, Arc<SRICompilationContext>>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug)]
pub struct SubresourceIntegrityPlugin {
  pub options: SubresourceIntegrityPluginOptions,
  pub validate_error: Option<rspack_error::Error>,
}

impl SubresourceIntegrityPlugin {
  pub fn new(
    options: SubresourceIntegrityPluginOptions,
    validate_error: Option<rspack_error::Error>,
  ) -> Self {
    Self::new_inner(options, validate_error)
  }

  pub fn get_compilation_sri_context(id: CompilationId) -> Arc<SRICompilationContext> {
    COMPILATION_CONTEXT_MAP
      .get(&id)
      .expect("should have sri context")
      .clone()
  }

  pub fn set_compilation_sri_context(id: CompilationId, ctx: SRICompilationContext) {
    COMPILATION_CONTEXT_MAP.insert(id, Arc::new(ctx));
  }

  pub fn get_compilation_integrities(id: CompilationId) -> Arc<RwLock<HashMap<String, String>>> {
    if !COMPILATION_INTEGRITY_MAP.contains_key(&id) {
      COMPILATION_INTEGRITY_MAP.insert(id, Default::default());
    }
    COMPILATION_INTEGRITY_MAP
      .get(&id)
      .expect("should have compilation integrities")
      .clone()
  }

  pub fn get_compilation_integrities_mut(
    id: CompilationId,
  ) -> Arc<RwLock<HashMap<String, String>>> {
    COMPILATION_INTEGRITY_MAP.entry(id).or_default().clone()
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
async fn validate_error(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.push_diagnostic(Diagnostic::error(
    "SubresourceIntegrity".to_string(),
    self
      .validate_error
      .as_ref()
      .expect("should have validate error")
      .to_string(),
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
    runtime_template: compilation.runtime_template.clone_without_dojang(),
  };
  SubresourceIntegrityPlugin::set_compilation_sri_context(compilation.id(), ctx);

  {
    let real_content_hash_plugin_hooks =
      RealContentHashPlugin::get_compilation_hooks_mut(compilation.id());
    let mut real_content_hash_plugin_hooks = real_content_hash_plugin_hooks.borrow_mut();
    real_content_hash_plugin_hooks
      .update_hash
      .tap(update_hash::new(self));
  }

  if matches!(
    compilation.options.output.cross_origin_loading,
    CrossOriginLoading::Disable
  ) {
    compilation.push_diagnostic(Diagnostic::error(
      "SubresourceIntegrity".to_string(),
      "Subresource integrity is not applied to async chunks because the \"output.crossOriginLoading\" option is not set.".to_string(),
    ));
  }

  {
    let runtime_plugin_hooks = RuntimePlugin::get_compilation_hooks_mut(compilation.id());
    let mut runtime_plugin_hooks = runtime_plugin_hooks.borrow_mut();
    runtime_plugin_hooks
      .create_script
      .tap(create_script::new(self));
    runtime_plugin_hooks.create_link.tap(create_link::new(self));
    runtime_plugin_hooks
      .link_preload
      .tap(link_preload::new(self));
  }

  if matches!(self.options.html_plugin, IntegrityHtmlPlugin::NativePlugin) {
    let html_plugin_hooks = HtmlRspackPlugin::get_compilation_hooks_mut(compilation.id());
    let mut html_plugin_hooks = html_plugin_hooks.borrow_mut();
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

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if self.validate_error.is_some() {
      ctx
        .compiler_hooks
        .this_compilation
        .tap(validate_error::new(self));
      return Ok(());
    }

    if let ChunkLoading::Enable(chunk_loading) = &ctx.compiler_options.output.chunk_loading
      && matches!(
        chunk_loading,
        ChunkLoadingType::Require | ChunkLoadingType::AsyncNode
      )
    {
      ctx
        .compiler_hooks
        .this_compilation
        .tap(warn_non_web::new(self));

      return Ok(());
    }

    ctx
      .compilation_hooks
      .process_assets
      .tap(handle_assets::new(self));

    ctx
      .compilation_hooks
      .after_process_assets
      .tap(detect_unresolved_integrity::new(self));

    ctx
      .compiler_hooks
      .this_compilation
      .tap(handle_compilation::new(self));

    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(handle_runtime::new(self));
    Ok(())
  }
}
