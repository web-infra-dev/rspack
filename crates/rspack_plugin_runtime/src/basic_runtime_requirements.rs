use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext, RuntimeGlobals, RuntimeModuleExt, SourceType,
};
use rspack_error::Result;

use crate::runtime_module::{
  CreateScriptUrlRuntimeModule, DefinePropertyGettersRuntimeModule, GetChunkFilenameRuntimeModule,
  GetChunkUpdateFilenameRuntimeModule, GetFullHashRuntimeModule, GetMainFilenameRuntimeModule,
  GetTrustedTypesPolicyRuntimeModule, GlobalRuntimeModule, HasOwnPropertyRuntimeModule,
  LoadChunkWithModuleRuntimeModule, LoadScriptRuntimeModule, NormalRuntimeModule,
  OnChunkLoadedRuntimeModule, PublicPathRuntimeModule,
};

#[derive(Debug)]
pub struct BasicRuntimeRequirementPlugin;

#[async_trait]
impl Plugin for BasicRuntimeRequirementPlugin {
  fn name(&self) -> &'static str {
    "BasicRuntimeRequirementPlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
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

    if runtime_requirements.contains(RuntimeGlobals::INTEROP_REQUIRE) {
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          RuntimeGlobals::INTEROP_REQUIRE,
          include_str!("runtime/common/_interop_require.js"),
        )
        .boxed(),
      )
    }

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

    let mut sorted_runtime_requirement = runtime_requirements.iter().collect::<Vec<_>>();
    // TODO: we don't need sort since iter is deterministic for BitFlags
    sorted_runtime_requirement.sort_unstable_by_key(|r| r.name());
    for runtime_requirement in sorted_runtime_requirement {
      match runtime_requirement {
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
        _ => {}
      }
    }

    Ok(())
  }
}
