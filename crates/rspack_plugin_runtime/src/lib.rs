use anyhow::anyhow;
use async_trait::async_trait;
use common::*;
use node::*;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModule, SourceType,
  TargetPlatform,
};
use rspack_error::Result;
use web::*;
use web_worker::*;

mod common;
mod node;
mod web;
mod web_worker;

pub const RUNTIME_FILE_NAME: &str = "runtime";
pub const RSPACK_REQUIRE: &str = "__rspack_require__";
pub const RSPACK_DYNAMIC_IMPORT: &str = "__rspack_dynamic_require__";
pub const RSPACK_REGISTER: &str = "__rspack_register__";
pub const RSPACK_RUNTIME: &str = "__rspack_runtime__";

#[derive(Debug)]
pub struct ChunkHash {
  name: String,
  hash: Option<String>,
}
#[derive(Debug)]
pub struct RuntimePlugin {}

impl RuntimePlugin {
  fn process_assets_with_less_runtime() {}
}

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
    let runtime_requirements = &args.runtime_requirements;
    let namespace = compilation.options.output.unique_name.clone();
    let public_path = compilation
      .options
      .output
      .public_path
      .public_path()
      .to_string();

    //Todo we are not implement hash now，it will be replaced by real value later
    let has_hash = false;

    let mut dynamic_js: Vec<ChunkHash> = vec![];
    let mut dynamic_css: Vec<ChunkHash> = vec![];
    let mut chunks = compilation.chunk_by_ukey.values().collect::<Vec<_>>();
    chunks.sort_by_key(|c| &c.id);
    for chunk in &chunks {
      if !chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
        dynamic_js.push(ChunkHash {
          name: chunk.id.clone(),
          hash: None,
        });

        let modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
          &chunk.ukey,
          SourceType::Css,
          &compilation.module_graph,
        );
        if !modules.is_empty() {
          dynamic_css.push(ChunkHash {
            name: chunk.id.clone(),
            hash: None,
          });
        }
      }
    }

    match &compilation.options.target.platform {
      TargetPlatform::Web => {
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_init_runtime.js".to_string(),
            generate_common_init_runtime(&namespace),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("_module_and_chunk_data.js").to_string(),
            generate_common_module_and_chunk_data(),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("_check_by_id.js").to_string(),
            generate_common_check_by_id(),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("_public_path.js").to_string(),
            generate_common_public_path(&public_path),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("_rspack_require.js").to_string(),
            generate_web_rspack_require(),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("_rspack_register.js").to_string(),
            generate_web_rspack_register(),
          ),
        );

        // TODO: should use `.hmrF = [chunk_id].[hash].hot-update.json`
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("__rspack_require__.chunkId").to_string(),
            format!(
              "(function(){{\nruntime.__rspack_require__.chunkId = '{}'}})();",
              compilation
                .chunk_by_ukey
                .get(chunk)
                .ok_or_else(|| anyhow!("chunk should exsit in chunk_by_ukey"))?
                .id,
            ),
          ),
        );

        // publicPath
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            ("__rspack_require__.p").to_string(),
            format!(
              "(function(){{\nruntime.__rspack_require__.p = '{}'}})();",
              compilation.options.output.public_path.public_path(),
            ),
          ),
        );

        if compilation.options.dev_server.hot {
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(("_hot.js").to_string(), generate_web_hot()),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              ("_load_script_content.js").to_string(),
              generate_web_load_script_content(),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(("_jsonp.js").to_string(), generate_web_jsonp()),
          );
        }

        if !dynamic_js.is_empty() || !dynamic_css.is_empty() {
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_data.js".to_string(),
              generate_common_dynamic_data(dynamic_js, dynamic_css),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_get_chunk_url.js".to_string(),
              generate_web_dynamic_get_chunk_url(has_hash),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_require.js".to_string(),
              generate_web_dynamic_require(),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_load_script.js".to_string(),
              generate_web_dynamic_load_script(),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_load_style.js".to_string(),
              generate_web_dynamic_load_style(),
            ),
          );
        }
      }
      TargetPlatform::WebWorker => {
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_init_runtime.js".to_string(),
            generate_web_worker_init_runtime(&namespace),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_init_runtime.js".to_string(),
            generate_web_worker_init_runtime(&namespace),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_module_and_chunk_data.js".to_string(),
            generate_common_module_and_chunk_data(),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new("_check_by_id.js".to_string(), generate_common_check_by_id()),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_rspack_require.js".to_string(),
            generate_web_rspack_require(),
          ),
        );
      }
      TargetPlatform::Node(_) => {
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_init_runtime.js".to_string(),
            generate_node_init_runtime(&namespace),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_module_and_chunk_data.js".to_string(),
            generate_common_module_and_chunk_data(),
          ),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new("_check_by_id.js".to_string(), generate_common_check_by_id()),
        );
        compilation.add_runtime_module(
          chunk,
          RuntimeModule::new(
            "_rspack_require.js".to_string(),
            generate_node_rspack_require(),
          ),
        );
        if !dynamic_js.is_empty() || !dynamic_css.is_empty() {
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_data.js".to_string(),
              generate_common_dynamic_data(dynamic_js, dynamic_css),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_get_chunk_url.js".to_string(),
              generate_node_dynamic_get_chunk_url(has_hash),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_load_chunk.js".to_string(),
              generate_node_load_chunk(),
            ),
          );
          compilation.add_runtime_module(
            chunk,
            RuntimeModule::new(
              "_dynamic_require.js".to_string(),
              generate_node_dynamic_require(),
            ),
          );
        }
      }
      _ => {}
    }

    if runtime_requirements.contains(runtime_globals::INTEROP_REQUIRE) {
      compilation.add_runtime_module(
        chunk,
        RuntimeModule::new(
          runtime_globals::INTEROP_REQUIRE.to_string(),
          include_str!("runtime/common/_interop_require.js").to_string(),
        ),
      )
    }

    if runtime_requirements.contains(runtime_globals::EXPORT_STAR) {
      compilation.add_runtime_module(
        chunk,
        RuntimeModule::new(
          runtime_globals::EXPORT_STAR.to_string(),
          include_str!("runtime/common/_export_star.js").to_string(),
        ),
      )
    }

    Ok(())
  }
}
