use std::hash::Hash;

use rspack_core::{
  ChunkCodeTemplate, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilerCompilation, Filename, LibraryName, LibraryNonUmdObject,
  LibraryOptions, LibraryType, PathData, Plugin, RuntimeGlobals, RuntimeModule, SourceType,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRender, JsPlugin, RenderSource,
};

use crate::utils::{COMMON_LIBRARY_NAME_MESSAGE, get_options_for_chunk};

const PLUGIN_NAME: &str = "rspack.JsonpLibraryPlugin";

#[derive(Debug)]
struct JsonpLibraryPluginParsed<'a> {
  name: &'a str,
}

#[plugin]
#[derive(Debug)]
pub struct JsonpLibraryPlugin {
  library_type: LibraryType,
}

impl JsonpLibraryPlugin {
  pub fn new(library_type: LibraryType) -> Self {
    Self::new_inner(library_type)
  }

  fn parse_options<'a>(&self, library: &'a LibraryOptions) -> Result<JsonpLibraryPluginParsed<'a>> {
    match &library.name {
      Some(LibraryName::NonUmdObject(LibraryNonUmdObject::String(s))) => {
        Ok(JsonpLibraryPluginParsed { name: s.as_str() })
      }
      _ => {
        error_bail!("Jsonp library name must be a simple string. {COMMON_LIBRARY_NAME_MESSAGE}")
      }
    }
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &'a ChunkUkey,
  ) -> Result<Option<JsonpLibraryPluginParsed<'a>>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == self.library_type)
      .map(|library| self.parse_options(library))
      .transpose()
  }
}

#[plugin_hook(CompilerCompilation for JsonpLibraryPlugin)]
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

#[plugin_hook(JavascriptModulesRender for JsonpLibraryPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
  _runtime_template: &ChunkCodeTemplate,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let name = compilation
    .get_path(
      &Filename::from(options.name),
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
  let mut source = ConcatSource::default();
  source.add(RawStringSource::from(format!("{name}(")));
  source.add(render_source.source.clone());
  source.add(RawStringSource::from_static("\n)"));
  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for JsonpLibraryPlugin)]
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
  options.name.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for JsonpLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
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

impl Plugin for JsonpLibraryPlugin {
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
