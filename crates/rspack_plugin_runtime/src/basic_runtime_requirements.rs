use crate::runtime_module::{
  GetChunkFilenameRuntimeModule, GetChunkUpdateFilenameRuntimeModule, GetMainFilenameRuntimeModule,
  HasOwnPropertyRuntimeModule, LoadScriptRuntimeModule, PublicPathRuntimeModule,
};
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, NormalRuntimeModule, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt, SourceType,
};
use rspack_error::Result;

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
          runtime_globals::INTEROP_REQUIRE.to_string(),
          include_str!("runtime/common/_interop_require.js").to_string(),
        )
        .boxed(),
      )
    }

    if runtime_requirements.contains(runtime_globals::EXPORT_STAR) {
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          runtime_globals::EXPORT_STAR.to_string(),
          include_str!("runtime/common/_export_star.js").to_string(),
        )
        .boxed(),
      )
    }

    let mut sorted_runtime_requirement = runtime_requirements.iter().collect::<Vec<_>>();
    sorted_runtime_requirement.sort();
    for runtime_requirement in sorted_runtime_requirement.iter() {
      match runtime_requirement.as_str() {
        runtime_globals::PUBLIC_PATH => {
          compilation.add_runtime_module(chunk, PublicPathRuntimeModule::default().boxed())
        }
        runtime_globals::GET_CHUNK_SCRIPT_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "js".to_string(),
            SourceType::JavaScript,
            runtime_globals::GET_CHUNK_SCRIPT_FILENAME.to_string(),
            false,
          )
          .boxed(),
        ),
        runtime_globals::GET_CHUNK_CSS_FILENAME => compilation.add_runtime_module(
          chunk,
          GetChunkFilenameRuntimeModule::new(
            "css".to_string(),
            SourceType::Css,
            runtime_globals::GET_CHUNK_CSS_FILENAME.to_string(),
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
        _ => {}
      }
    }

    Ok(())
  }
}
