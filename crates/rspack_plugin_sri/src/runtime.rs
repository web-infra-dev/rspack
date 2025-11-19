use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements, CrossOriginLoading,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt, chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rspack_error::{Result, error};
use rspack_hook::plugin_hook;
use rspack_plugin_runtime::{
  CreateScriptData, LinkPreloadData, RuntimePluginCreateScript, RuntimePluginLinkPreload,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  SubresourceIntegrityHashFunction, SubresourceIntegrityPlugin, SubresourceIntegrityPluginInner,
  util::{SRI_HASH_VARIABLE_REFERENCE, find_chunks, make_placeholder},
};

fn add_attribute(tag: &str, code: &str, cross_origin_loading: &CrossOriginLoading) -> String {
  format!(
    "{}\n{tag}.integrity = {}[chunkId];\n{tag}.crossOrigin = {};",
    code, SRI_HASH_VARIABLE_REFERENCE, cross_origin_loading
  )
}

#[impl_runtime_module]
#[derive(Debug)]
struct SRIHashVariableRuntimeModule {
  id: Identifier,
  chunk: ChunkUkey,
  hash_funcs: Vec<SubresourceIntegrityHashFunction>,
}

impl SRIHashVariableRuntimeModule {
  pub fn new(chunk: ChunkUkey, hash_funcs: Vec<SubresourceIntegrityHashFunction>) -> Self {
    Self::with_default(
      Identifier::from("rspack/runtime/sri_hash_variable"),
      chunk,
      hash_funcs,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for SRIHashVariableRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    let Some(chunk) = compilation.chunk_by_ukey.get(&self.chunk) else {
      return Err(error!(
        "Generate sri runtime module failed: chunk not found"
      ));
    };

    let include_chunks = chunk
      .get_all_async_chunks(&compilation.chunk_group_by_ukey)
      .iter()
      .filter_map(|c| {
        let chunk = compilation.chunk_by_ukey.get(c)?;
        let id = chunk.id(&compilation.chunk_ids_artifact)?;
        let rendered_hash = chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        )?;
        Some((id, rendered_hash))
      })
      .collect::<HashMap<_, _>>();

    let module_graph = compilation.get_module_graph();
    let all_chunks = find_chunks(&self.chunk, compilation)
      .into_iter()
      .filter_map(|c| {
        let chunk = compilation.chunk_by_ukey.get(&c)?;
        let id = chunk.id(&compilation.chunk_ids_artifact)?;
        let has_modules = compilation
          .chunk_graph
          .get_chunk_modules(&c, &module_graph)
          .iter()
          .any(|m| m.source().is_some());

        if has_modules && include_chunks.contains_key(id) {
          Some(id)
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    Ok(format!(
      r#"
        {} = {};
        "#,
      SRI_HASH_VARIABLE_REFERENCE,
      generate_sri_hash_placeholders(all_chunks, &self.hash_funcs, compilation),
    ))
  }
}

fn generate_sri_hash_placeholders(
  chunks: Vec<&ChunkId>,
  hash_funcs: &Vec<SubresourceIntegrityHashFunction>,
  _compilation: &Compilation,
) -> String {
  format!(
    "{{{}}}",
    chunks
      .into_iter()
      .filter_map(|c| {
        let chunk_id = serde_json::to_string(c.as_str()).ok()?;
        let placeholder = serde_json::to_string(&make_placeholder(hash_funcs, c.as_str())).ok()?;
        Some(format!("{chunk_id}: {placeholder}"))
      })
      .collect::<Vec<_>>()
      .join(",")
  )
}

#[plugin_hook(RuntimePluginCreateScript for SubresourceIntegrityPlugin)]
pub async fn create_script(&self, mut data: CreateScriptData) -> Result<CreateScriptData> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.chunk.compilation_id);
  data.code = add_attribute("script", &data.code, &ctx.cross_origin_loading);
  Ok(data)
}

#[plugin_hook(RuntimePluginLinkPreload for SubresourceIntegrityPlugin)]
pub async fn link_preload(&self, mut data: LinkPreloadData) -> Result<LinkPreloadData> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.chunk.compilation_id);
  data.code = add_attribute("link", &data.code, &ctx.cross_origin_loading);
  Ok(data)
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for SubresourceIntegrityPlugin)]
pub async fn handle_runtime(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  compilation.add_runtime_module(
    chunk_ukey,
    SRIHashVariableRuntimeModule::new(*chunk_ukey, self.options.hash_func_names.clone()).boxed(),
  )?;
  Ok(())
}
