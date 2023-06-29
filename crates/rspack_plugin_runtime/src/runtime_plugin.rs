use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkLoading, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  RuntimeGlobals, RuntimeModuleExt, SourceType,
};
use rspack_error::Result;

use crate::runtime_module::{
  is_enabled_for_chunk, AsyncRuntimeModule, BaseUriRuntimeModule,
  CompatGetDefaultExportRuntimeModule, CreateFakeNamespaceObjectRuntimeModule,
  CreateScriptUrlRuntimeModule, DefinePropertyGettersRuntimeModule, EnsureChunkRuntimeModule,
  GetChunkFilenameRuntimeModule, GetChunkUpdateFilenameRuntimeModule, GetFullHashRuntimeModule,
  GetMainFilenameRuntimeModule, GetTrustedTypesPolicyRuntimeModule, GlobalRuntimeModule,
  HasOwnPropertyRuntimeModule, LoadChunkWithModuleRuntimeModule, LoadScriptRuntimeModule,
  MakeNamespaceObjectRuntimeModule, NormalRuntimeModule, OnChunkLoadedRuntimeModule,
  PublicPathRuntimeModule,
};

#[derive(Debug)]
pub struct RuntimePlugin;

#[async_trait]
impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "RuntimePlugin"
  }

  fn apply(&self, _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    if !chunk
      .get_all_async_chunks(&compilation.chunk_group_by_ukey)
      .is_empty()
      || args
        .runtime_requirements
        .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
    {
      args
        .runtime_requirements
        .insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    }

    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(RuntimeGlobals::EXPORT_STAR) {
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          RuntimeGlobals::EXPORT_STAR,
          include_str!("runtime/common/_export_star.js"),
        )
        .boxed(),
      )
    }

    if compilation.options.output.trusted_types.is_some() {
      if runtime_requirements.contains(RuntimeGlobals::LOAD_SCRIPT) {
        runtime_requirements.insert(RuntimeGlobals::CREATE_SCRIPT_URL);
      }
      if runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT)
        || runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT_URL)
      {
        runtime_requirements.insert(RuntimeGlobals::GET_TRUSTED_TYPES_POLICY);
      }
    }

    if (runtime_requirements.contains(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME)
      && compilation
        .options
        .output
        .hot_update_chunk_filename
        .has_hash_placeholder())
      || (runtime_requirements.contains(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME)
        && compilation
          .options
          .output
          .hot_update_main_filename
          .has_hash_placeholder())
    {
      runtime_requirements.insert(RuntimeGlobals::GET_FULL_HASH);
    }

    if runtime_requirements.contains(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT) {
      runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    }

    if runtime_requirements.contains(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT) {
      runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    }

    if runtime_requirements.contains(RuntimeGlobals::DEFINE_PROPERTY_GETTERS) {
      runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    }

    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ASYNC_MODULE => {
          compilation.add_runtime_module(chunk, AsyncRuntimeModule::default().boxed());
        }
        RuntimeGlobals::BASE_URI
          if is_enabled_for_chunk(chunk, &ChunkLoading::False, compilation) =>
        {
          compilation.add_runtime_module(chunk, BaseUriRuntimeModule::default().boxed());
        }
        RuntimeGlobals::ENSURE_CHUNK => {
          compilation.add_runtime_module(chunk, EnsureChunkRuntimeModule::new(true).boxed());
        }
        RuntimeGlobals::PUBLIC_PATH => {
          compilation.add_runtime_module(chunk, PublicPathRuntimeModule::default().boxed())
        }
        RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "js",
            SourceType::JavaScript,
            RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME,
            false,
          )
          .boxed(),
        ),
        RuntimeGlobals::GET_CHUNK_CSS_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "css",
            SourceType::Css,
            RuntimeGlobals::GET_CHUNK_CSS_FILENAME,
            runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS),
          )
          .boxed(),
        ),
        RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => {
          compilation.add_runtime_module(
            chunk,
            GetChunkUpdateFilenameRuntimeModule::default().boxed(),
          );
        }
        RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => compilation.add_runtime_module(
          chunk,
          GetMainFilenameRuntimeModule::new(
            RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME,
            compilation
              .options
              .output
              .hot_update_main_filename
              .template()
              .to_string(),
          )
          .boxed(),
        ),
        RuntimeGlobals::LOAD_SCRIPT => compilation.add_runtime_module(
          chunk,
          LoadScriptRuntimeModule::new(compilation.options.output.trusted_types.is_some()).boxed(),
        ),
        RuntimeGlobals::HAS_OWN_PROPERTY => {
          compilation.add_runtime_module(chunk, HasOwnPropertyRuntimeModule::default().boxed())
        }
        RuntimeGlobals::GET_FULL_HASH => {
          compilation.add_runtime_module(chunk, GetFullHashRuntimeModule::default().boxed())
        }
        RuntimeGlobals::LOAD_CHUNK_WITH_MODULE => {
          compilation.add_runtime_module(chunk, LoadChunkWithModuleRuntimeModule::default().boxed())
        }
        RuntimeGlobals::GLOBAL => {
          compilation.add_runtime_module(chunk, GlobalRuntimeModule::default().boxed())
        }
        RuntimeGlobals::CREATE_SCRIPT_URL => {
          compilation.add_runtime_module(chunk, CreateScriptUrlRuntimeModule::default().boxed())
        }
        RuntimeGlobals::ON_CHUNKS_LOADED => {
          compilation.add_runtime_module(chunk, OnChunkLoadedRuntimeModule::default().boxed());
        }
        RuntimeGlobals::DEFINE_PROPERTY_GETTERS => compilation
          .add_runtime_module(chunk, DefinePropertyGettersRuntimeModule::default().boxed()),
        RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => compilation
          .add_runtime_module(chunk, GetTrustedTypesPolicyRuntimeModule::default().boxed()),
        RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => compilation.add_runtime_module(
          chunk,
          CreateFakeNamespaceObjectRuntimeModule::default().boxed(),
        ),
        RuntimeGlobals::MAKE_NAMESPACE_OBJECT => {
          compilation.add_runtime_module(chunk, MakeNamespaceObjectRuntimeModule::default().boxed())
        }
        RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => compilation.add_runtime_module(
          chunk,
          CompatGetDefaultExportRuntimeModule::default().boxed(),
        ),
        _ => {}
      }
    }

    Ok(())
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    for identifier in args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(args.chunk_ukey)
    {
      if let Some((hash, _)) = args
        .compilation
        .runtime_module_code_generation_results
        .get(identifier)
      {
        hash.hash(&mut args.hasher);
      }
    }
    Ok(())
  }
}
