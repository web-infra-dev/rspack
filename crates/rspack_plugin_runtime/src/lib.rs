use async_trait::async_trait;
use rspack_error::Result;

use common::*;
use rspack_core::{
  AssetContent, ChunkKind, Plugin, PluginContext, PluginRenderManifestHookOutput,
  PluginRenderRuntimeHookOutput, RenderManifestArgs, RenderManifestEntry, RenderRuntimeArgs,
  RuntimeSourceNode, TargetPlatform, RUNTIME_PLACEHOLDER_RSPACK_EXECUTE,
};
use web::*;
use web_worker::*;

mod common;
mod web;
mod web_worker;

pub const RUNTIME_FILE_NAME: &str = "runtime";

#[derive(Debug)]
pub struct ChunkHash {
  name: String,
  hash: Option<String>,
}
#[derive(Debug)]
pub struct RuntimePlugin {}

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

  fn render_runtime(
    &self,
    _ctx: PluginContext,
    args: RenderRuntimeArgs,
  ) -> PluginRenderRuntimeHookOutput {
    let compilation = args.compilation;
    let public_path = compilation.options.output.public_path.public_path();

    //Todo we are not implement hash now，it will be replaced by real value later
    let has_hash = false;

    let mut dynamic_js: Vec<ChunkHash> = vec![];
    let mut dynamic_css: Vec<ChunkHash> = vec![];
    for (_, chunk) in &compilation.chunk_by_ukey {
      if matches!(chunk.kind, ChunkKind::Normal) {
        for file in &chunk.files {
          if file.ends_with(".js") && !file.eq(&(RUNTIME_FILE_NAME.to_string() + ".js")) {
            dynamic_js.push(ChunkHash {
              name: chunk.id.clone(),
              hash: None,
            });
          } else if file.ends_with(".css") {
            dynamic_css.push(ChunkHash {
              name: chunk.id.clone(),
              hash: None,
            });
          }
        }
      }
    }
    // if the complition has dynamic chunk
    //Todo we need a dynamic chunk tag to judge it

    // common runtime
    let mut sources = args.sources.to_vec();

    match &compilation.options.target.platform {
      TargetPlatform::Web => {
        sources.push(generate_common_init_runtime());
        sources.push(generate_common_module_and_chunk_data());
        sources.push(generate_common_check_by_id());
        sources.push(generate_common_public_path(public_path));
        sources.push(generate_web_rspack_require());
        sources.push(generate_web_rspack_register());
        if !dynamic_js.is_empty() || !dynamic_css.is_empty() {
          sources.push(generate_common_dynamic_data(dynamic_js, dynamic_css));
          sources.push(generate_web_dynamic_get_chunk_url(has_hash));
          sources.push(generate_web_dynamic_require());
          sources.push(generate_web_dynamic_load_script());
          sources.push(generate_web_dynamic_load_style());
        }
      }
      TargetPlatform::WebWorker => {
        sources.push(generate_web_worker_init_runtime());
        sources.push(generate_common_module_and_chunk_data());
        sources.push(generate_common_check_by_id());
        sources.push(generate_web_rspack_require());
        sources.push(RuntimeSourceNode {
          content: RUNTIME_PLACEHOLDER_RSPACK_EXECUTE.to_string(),
        });
      }
      _ => {}
    }
    Ok(sources)
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    //Todo we need add optimize.runtime to ensure runtime generation
    if matches!(
      compilation.options.target.platform,
      TargetPlatform::WebWorker,
    ) {
      Ok(vec![])
    } else {
      let compilation = args.compilation;
      let runtime = &compilation.runtime;
      Ok(vec![RenderManifestEntry::new(
        AssetContent::String(runtime.generate()),
        RUNTIME_FILE_NAME.to_string() + ".js",
      )])
    }
  }

  async fn process_assets(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let runtime = &compilation.runtime;
    compilation.emit_asset(
      RUNTIME_FILE_NAME.to_string() + ".js",
      rspack_core::CompilationAsset {
        source: AssetContent::String(runtime.generate()),
      },
    );
    Ok(())
  }
}
