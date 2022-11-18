use crate::runtime_module::{
  EnsureChunkRuntimeModule, GetChunkFilenameRuntimeModule, HasOwnPropertyRuntimeModule,
  LoadScriptRuntimeModule, OnChunkLoadedRuntimeModule, PublicPathRuntimeModule,
};
use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, NormalRuntimeModule, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt, TargetPlatform,
};
use rspack_error::Result;
use web::*;

mod array_push_callback_chunk_format;
pub use array_push_callback_chunk_format::ArrayPushCallbackChunkFormatPlugin;
mod common_js_chunk_loading;
pub use common_js_chunk_loading::CommonJsChunkLoadingPlugin;
mod jsonp_chunk_loading;
pub use jsonp_chunk_loading::JsonPChunkLoadingPlugin;
mod runtime_module;
mod web;

#[derive(Debug)]
pub struct RuntimePlugin {}

impl RuntimePlugin {}

#[async_trait]
impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "runtime"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    let mut chunks = compilation.chunk_by_ukey.values().collect::<Vec<_>>();
    chunks.sort_by_key(|c| &c.id);
    for chunk in &chunks {
      if !chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
        // TODO: use module async block instead of it at code generation
        runtime_requirements.insert(runtime_globals::ENSURE_CHUNK.to_string());
      }
    }

    compilation.add_runtime_module(
      chunk,
      NormalRuntimeModule::new(
        ("_rspack_require.js").to_string(),
        include_str!("runtime/require.js").to_string().replace(
          "GLOBAL",
          if matches!(
            compilation.options.target.platform,
            TargetPlatform::Web | TargetPlatform::None
          ) {
            "self"
          } else {
            "this"
          },
        ),
      )
      .boxed(),
    );

    if compilation.options.dev_server.hot
      && matches!(compilation.options.target.platform, TargetPlatform::Web)
    {
      // TODO: should use `.hmrF = [chunk_id].[hash].hot-update.json`
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          ("n__webpack_require__.chunkId").to_string(),
          format!(
            "(function(){{\n__webpack_require__.chunkId = '{}'}})();",
            compilation
              .chunk_by_ukey
              .get(chunk)
              .ok_or_else(|| anyhow!("chunk should exsit in chunk_by_ukey"))?
              .id,
          ),
        )
        .boxed(),
      );

      // TODO: need hash
      // hu: get the filename of hotUpdateChunk.
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(
          ("n__webpack_require__.hu").to_string(),
          r#"
            (function(){
              __webpack_require__.hu = function (chunkId) {
                return '' + chunkId + '.hot-update.js';
              }
            })();"#
            .to_string(),
        )
        .boxed(),
      );
      runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY.to_string());
      runtime_requirements.insert(runtime_globals::PUBLIC_PATH.to_string());
      runtime_requirements.insert(runtime_globals::LOAD_SCRIPT.to_string());
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(("_hot.js").to_string(), generate_web_hot()).boxed(),
      );
      compilation.add_runtime_module(
        chunk,
        NormalRuntimeModule::new(("_jsonp.js").to_string(), generate_web_jsonp()).boxed(),
      );
    }

    // workaround for jsonp_chunk_loading can scan `ENSURE_CHUNK` to add additional runtime_requirements
    if runtime_requirements.contains(runtime_globals::ENSURE_CHUNK) {
      runtime_requirements.insert(runtime_globals::ENSURE_CHUNK_HANDLERS.to_string());
      compilation.add_runtime_module(chunk, EnsureChunkRuntimeModule::new(true).boxed());
    }

    if runtime_requirements.contains(runtime_globals::ON_CHUNKS_LOADED) {
      compilation.add_runtime_module(chunk, OnChunkLoadedRuntimeModule::default().boxed());
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
          GetChunkFilenameRuntimeModule::new("js".to_string()).boxed(),
        ),
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
