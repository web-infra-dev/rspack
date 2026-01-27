use std::hash::Hash;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements, CompilationParams,
  CompilerCompilation, ExternalModule, Filename, LibraryName, LibraryNonUmdObject, LibraryOptions,
  LibraryType, PathData, Plugin, RuntimeGlobals, SourceType,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRender, JsPlugin, RenderSource,
};

use crate::utils::{
  COMMON_LIBRARY_NAME_MESSAGE, external_arguments, externals_dep_array, get_options_for_chunk,
};

const PLUGIN_NAME: &str = "rspack.AmdLibraryPlugin";

#[derive(Debug)]
struct AmdLibraryPluginParsed<'a> {
  name: Option<&'a str>,
  amd_container: Option<&'a str>,
}

#[plugin]
#[derive(Debug)]
pub struct AmdLibraryPlugin {
  require_as_wrapper: bool,
  library_type: LibraryType,
}

impl AmdLibraryPlugin {
  pub fn new(require_as_wrapper: bool, library_type: LibraryType) -> Self {
    Self::new_inner(require_as_wrapper, library_type)
  }

  fn parse_options<'a>(&self, library: &'a LibraryOptions) -> Result<AmdLibraryPluginParsed<'a>> {
    if self.require_as_wrapper {
      if library.name.is_some() {
        error_bail!("AMD library name must be unset. {COMMON_LIBRARY_NAME_MESSAGE}")
      }
    } else if let Some(name) = &library.name
      && !matches!(
        name,
        LibraryName::NonUmdObject(LibraryNonUmdObject::String(_))
      )
    {
      error_bail!(
        "AMD library name must be a simple string or unset. {COMMON_LIBRARY_NAME_MESSAGE}"
      )
    }
    Ok(AmdLibraryPluginParsed {
      name: library.name.as_ref().map(|name| match name {
        LibraryName::NonUmdObject(LibraryNonUmdObject::String(s)) => s.as_str(),
        _ => unreachable!("AMD library name must be a simple string or unset."),
      }),
      amd_container: library.amd_container.as_deref(),
    })
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &'a ChunkUkey,
  ) -> Result<Option<AmdLibraryPluginParsed<'a>>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == self.library_type)
      .map(|library| self.parse_options(library))
      .transpose()
  }
}

#[plugin_hook(CompilerCompilation for AmdLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render.tap(render::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRender for AmdLibraryPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();
  let modules = compilation
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
    .iter()
    .filter_map(|identifier| {
      module_graph
        .module_by_identifier(identifier)
        .and_then(|module| module.as_external_module())
        .and_then(|m| {
          let ty = m.get_external_type();
          (ty == "amd" || ty == "amd-require").then_some(m)
        })
    })
    .collect::<Vec<&ExternalModule>>();
  let externals_deps_array = externals_dep_array(&modules)?;
  let external_arguments = external_arguments(&modules, compilation);
  let mut fn_start = format!("function({external_arguments}){{\n");
  if compilation.options.output.iife || !chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    fn_start.push_str(" return ");
  }
  let mut source = ConcatSource::default();
  let amd_container_prefix = options
    .amd_container
    .map(|c| format!("{c}."))
    .unwrap_or_default();
  if self.require_as_wrapper {
    source.add(RawStringSource::from(format!(
      "{amd_container_prefix}require({externals_deps_array}, {fn_start}"
    )));
  } else if let Some(name) = options.name {
    let normalize_name = compilation
      .get_path(
        &Filename::from(name),
        PathData::default()
          .chunk_id_optional(chunk.id().map(|id| id.as_str()))
          .chunk_hash_optional(chunk.rendered_hash(
            &compilation.chunk_hashes_artifact,
            compilation.options.output.hash_digest_length,
          ))
          .chunk_name_optional(chunk.name_for_filename_template())
          .content_hash_optional(chunk.rendered_content_hash_by_source_type(
            &compilation.chunk_hashes_artifact,
            &SourceType::JavaScript,
            compilation.options.output.hash_digest_length,
          )),
      )
      .await?;
    source.add(RawStringSource::from(format!(
      "{amd_container_prefix}define('{normalize_name}', {externals_deps_array}, {fn_start}"
    )));
  } else if modules.is_empty() {
    source.add(RawStringSource::from(format!(
      "{amd_container_prefix}define({fn_start}"
    )));
  } else {
    source.add(RawStringSource::from(format!(
      "{amd_container_prefix}define({externals_deps_array}, {fn_start}"
    )));
  }
  source.add(render_source.source.clone());
  source.add(RawStringSource::from_static("\n})"));
  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for AmdLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  PLUGIN_NAME.hash(hasher);
  if self.require_as_wrapper {
    self.require_as_wrapper.hash(hasher);
  } else if let Some(name) = options.name {
    "named".hash(hasher);
    name.hash(hasher);
  } else if let Some(amd_container) = options.amd_container {
    "amdContainer".hash(hasher);
    amd_container.hash(hasher);
  }
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for AmdLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if self
    .get_options_for_chunk(compilation, chunk_ukey)?
    .is_none()
  {
    return Ok(());
  }
  runtime_requirements.insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
  Ok(())
}

impl Plugin for AmdLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}
