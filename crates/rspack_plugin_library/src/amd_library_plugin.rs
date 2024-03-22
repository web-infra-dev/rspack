use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  AdditionalChunkRuntimeRequirementsArgs, ChunkUkey, Compilation, ExternalModule, FilenameTemplate,
  JsChunkHashArgs, LibraryName, LibraryNonUmdObject, LibraryOptions, LibraryType, PathData, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderHookOutput, RenderArgs, RuntimeGlobals, SourceType,
};
use rspack_error::{error_bail, Result};
use rspack_util::infallible::ResultInfallibleExt as _;

use crate::utils::{
  external_arguments, externals_dep_array, get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE,
};

#[derive(Debug)]
struct AmdLibraryPluginParsed<'a> {
  name: Option<&'a str>,
  amd_container: Option<&'a str>,
}

#[derive(Debug)]
pub struct AmdLibraryPlugin {
  library_type: LibraryType,
  require_as_wrapper: bool,
}

impl AmdLibraryPlugin {
  pub fn new(require_as_wrapper: bool, library_type: LibraryType) -> Self {
    Self {
      require_as_wrapper,
      library_type,
    }
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

#[async_trait::async_trait]
impl Plugin for AmdLibraryPlugin {
  fn name(&self) -> &'static str {
    "rspack.AmdLibraryPlugin"
  }

  async fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    if self
      .get_options_for_chunk(args.compilation, args.chunk)?
      .is_none()
    {
      return Ok(());
    }
    args
      .runtime_requirements
      .insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
    Ok(())
  }

  fn render(&self, _ctx: PluginContext, args: &RenderArgs) -> PluginRenderHookOutput {
    let compilation = args.compilation;
    let Some(options) = self.get_options_for_chunk(compilation, args.chunk)? else {
      return Ok(None);
    };
    let chunk = args.chunk();
    let modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(args.chunk)
      .iter()
      .filter_map(|identifier| {
        compilation
          .get_module_graph()
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
      source.add(RawSource::from(format!(
        "{amd_container_prefix}require({externals_deps_array}, {fn_start}"
      )));
    } else if let Some(name) = options.name {
      let normalize_name = compilation
        .get_path(
          &FilenameTemplate::from(name.to_string()),
          PathData::default().chunk(chunk).content_hash_optional(
            chunk
              .content_hash
              .get(&SourceType::JavaScript)
              .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
          ),
        )
        .always_ok();
      source.add(RawSource::from(format!(
        "{amd_container_prefix}define('{normalize_name}', {externals_deps_array}, {fn_start}"
      )));
    } else if modules.is_empty() {
      source.add(RawSource::from(format!(
        "{amd_container_prefix}define({fn_start}"
      )));
    } else {
      source.add(RawSource::from(format!(
        "{amd_container_prefix}define({externals_deps_array}, {fn_start}"
      )));
    }
    source.add(args.source.clone());
    source.add(RawSource::from("\n});"));
    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    let compilation = args.compilation;
    let Some(options) = self.get_options_for_chunk(compilation, args.chunk_ukey)? else {
      return Ok(());
    };
    self.name().hash(&mut args.hasher);
    if self.require_as_wrapper {
      self.require_as_wrapper.hash(&mut args.hasher);
    } else if let Some(name) = options.name {
      "named".hash(&mut args.hasher);
      name.hash(&mut args.hasher);
    } else if let Some(amd_container) = options.amd_container {
      "amdContainer".hash(&mut args.hasher);
      amd_container.hash(&mut args.hasher);
    }
    Ok(())
  }
}
