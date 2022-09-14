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
pub const RSPACK_REQUIRE: &str = "__rspack_require__";
pub const RSPACK_DYNAMIC_IMPORT: &str = "__rspack_dynamic_require__";
pub const RSPACK_REGISTER: &str = "__rspack_register__";

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
    let namespace = &compilation.options.output.unique_name;
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
        sources.push(generate_common_init_runtime(namespace));
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
      TargetPlatform::WebWorker | TargetPlatform::Node(_) => {
        sources.push(generate_web_worker_init_runtime(namespace));
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
    if TargetPlatform::is_web(&compilation.options.target.platform) {
      let compilation = args.compilation;
      let runtime = &compilation.runtime;
      Ok(vec![RenderManifestEntry::new(
        AssetContent::String(runtime.generate()),
        RUNTIME_FILE_NAME.to_string() + ".js",
      )])
    } else {
      Ok(vec![])
    }
  }

  async fn process_assets(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let namespace = &compilation.options.output.unique_name;
    let platform = &compilation.options.target.platform;

    match platform {
      TargetPlatform::WebWorker | TargetPlatform::Node(_) => {
        let mut entry_code_array: Vec<(String, String)> = vec![];
        let _ = &compilation.chunk_by_ukey.values().for_each(|chunk| {
          if matches!(chunk.kind, ChunkKind::Entry { .. }) {
            let js_entry_file = chunk
              .files
              .iter()
              .find(|file| file.ends_with(".js"))
              .unwrap();
            if let AssetContent::String(code) =
              &compilation.assets.get(js_entry_file).unwrap().source()
            {
              let ordered_modules = compilation
                .chunk_graph
                .get_chunk_modules(&chunk.ukey, &compilation.module_graph);
              let entry_module_id = ordered_modules.last().unwrap().id.as_str();
              let execute_code = compilation.runtime.generate_rspack_execute(
                namespace,
                RSPACK_REQUIRE,
                entry_module_id,
              );
              entry_code_array.push((
                js_entry_file.to_string(),
                compilation
                  .runtime
                  .generate_with_inline_modules(code, &execute_code),
              ));
            }
          }
        });
        for (file, code) in entry_code_array {
          compilation.emit_asset(
            file.to_string(),
            rspack_core::CompilationAsset {
              source: AssetContent::String(code),
            },
          );
        }
      }
      _ => {
        compilation.emit_asset(
          RUNTIME_FILE_NAME.to_string() + ".js",
          rspack_core::CompilationAsset {
            source: AssetContent::String(compilation.runtime.generate()),
          },
        );
      }
    }

    Ok(())
  }
}
