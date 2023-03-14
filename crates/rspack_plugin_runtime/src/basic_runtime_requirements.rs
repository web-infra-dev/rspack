use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt, SourceType,
};
use rspack_error::Result;

use crate::runtime_module::{
  GetChunkFilenameRuntimeModule, GetChunkUpdateFilenameRuntimeModule, GetFullHashRuntimeModule,
  GetMainFilenameRuntimeModule, GlobalRuntimeModule, HasOwnPropertyRuntimeModule,
  LoadChunkWithModuleRuntimeModule, LoadScriptRuntimeModule, NormalRuntimeModule,
  PublicPathRuntimeModule,
};

#[derive(Debug)]
pub struct BasicRuntimeRequirementPlugin {}

impl BasicRuntimeRequirementPlugin {}

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

    if runtime_requirements.contains(runtime_globals::INTEROP_REQUIRE) {
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          runtime_globals::INTEROP_REQUIRE,
          include_str!("runtime/common/_interop_require.js"),
        )
        .boxed(),
      )
    }

    if runtime_requirements.contains(runtime_globals::EXPORT_STAR) {
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          runtime_globals::EXPORT_STAR,
          include_str!("runtime/common/_export_star.js"),
        )
        .boxed(),
      )
    }

    let mut sorted_runtime_requirement = runtime_requirements.iter().collect::<Vec<_>>();
    sorted_runtime_requirement.sort_unstable();
    for &runtime_requirement in sorted_runtime_requirement {
      match runtime_requirement {
        runtime_globals::PUBLIC_PATH => {
          compilation.add_runtime_module(chunk, PublicPathRuntimeModule::default().boxed())
        }
        runtime_globals::GET_CHUNK_SCRIPT_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "js",
            SourceType::JavaScript,
            runtime_globals::GET_CHUNK_SCRIPT_FILENAME,
            false,
          )
          .boxed(),
        ),
        runtime_globals::GET_CHUNK_CSS_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "css",
            SourceType::Css,
            runtime_globals::GET_CHUNK_CSS_FILENAME,
            runtime_requirements.contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS),
          )
          .boxed(),
        ),
        runtime_globals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => {
          // TODO: need hash
          // hu: get the filename of hotUpdateChunk.
          compilation.add_runtime_module(
            chunk,
            GetChunkUpdateFilenameRuntimeModule::default().boxed(),
          );
        }
        runtime_globals::GET_UPDATE_MANIFEST_FILENAME => {
          compilation.add_runtime_module(chunk, GetMainFilenameRuntimeModule::default().boxed())
        }
        runtime_globals::LOAD_SCRIPT => {
          compilation.add_runtime_module(chunk, LoadScriptRuntimeModule::default().boxed())
        }
        runtime_globals::HAS_OWN_PROPERTY => {
          compilation.add_runtime_module(chunk, HasOwnPropertyRuntimeModule::default().boxed())
        }
        runtime_globals::GET_FULL_HASH => {
          compilation.add_runtime_module(chunk, GetFullHashRuntimeModule::default().boxed())
        }
        runtime_globals::LOAD_CHUNK_WITH_MODULE => {
          compilation.add_runtime_module(chunk, LoadChunkWithModuleRuntimeModule::default().boxed())
        }
        runtime_globals::GLOBAL => {
          compilation.add_runtime_module(chunk, GlobalRuntimeModule::default().boxed())
        }
        _ => {}
      }
    }

    Ok(())
  }
}
