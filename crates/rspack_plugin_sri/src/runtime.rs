use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements, CrossOriginLoading,
  ManifestAssetType, RuntimeGlobals, RuntimeModule, RuntimeModuleExt, SourceType,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rspack_error::{Result, error};
use rspack_hook::plugin_hook;
use rspack_plugin_runtime::{
  CreateLinkData, CreateScriptData, LinkPreloadData, RuntimePluginCreateLink,
  RuntimePluginCreateScript, RuntimePluginLinkPreload,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  SubresourceIntegrityHashFunction, SubresourceIntegrityPlugin, SubresourceIntegrityPluginInner,
  util::{find_chunks, get_hash_varaiable, make_placeholder},
};

fn add_attribute(
  tag: &str,
  variable_ref: &str,
  code: &str,
  cross_origin_loading: &CrossOriginLoading,
) -> String {
  format!(
    "{}\n{tag}.integrity = {}[chunkId];\n{tag}.crossOrigin = '{}';",
    code, variable_ref, cross_origin_loading
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

    let source_types = vec![
      (
        SourceType::JavaScript,
        get_hash_varaiable(&compilation.runtime_template, SourceType::JavaScript),
      ),
      (
        SourceType::Css,
        get_hash_varaiable(&compilation.runtime_template, SourceType::Css),
      ),
      (
        SourceType::Custom("css/mini-extract".into()),
        get_hash_varaiable(
          &compilation.runtime_template,
          SourceType::Custom("css/mini-extract".into()),
        ),
      ),
    ];

    let all_chunks = find_chunks(&self.chunk, compilation)
      .into_iter()
      .filter(|c| {
        compilation
          .chunk_graph
          .get_chunk_modules(c, &module_graph)
          .iter()
          .any(|m| m.source().is_some())
      })
      .collect::<Vec<_>>();

    let mut code = vec![];

    for (source_type, variable_ref) in source_types {
      let chunk_with_source_type = all_chunks
        .iter()
        .filter(|c| {
          compilation
            .chunk_graph
            .has_chunk_module_by_source_type(c, source_type, &module_graph)
        })
        .map(|c| {
          compilation
            .chunk_ids_artifact
            .get(c)
            .expect("should have chunk id in code generation")
        })
        .filter(|c| include_chunks.contains_key(c))
        .collect::<Vec<_>>();

      if !chunk_with_source_type.is_empty() {
        code.push(format!(
          r#"
          {} = {};
          "#,
          variable_ref,
          generate_sri_hash_placeholders(
            match source_type {
              SourceType::JavaScript => ManifestAssetType::JavaScript,
              SourceType::Css => ManifestAssetType::Css,
              SourceType::Custom(name) if name == "css/mini-extract" =>
                ManifestAssetType::Custom("extract-css".into()),
              _ => ManifestAssetType::Unknown,
            },
            chunk_with_source_type,
            &self.hash_funcs,
            compilation
          ),
        ));
      }
    }

    Ok(code.join("\n"))
  }
}

fn generate_sri_hash_placeholders(
  asset_type: ManifestAssetType,
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
        let placeholder =
          serde_json::to_string(&make_placeholder(asset_type, hash_funcs, c.as_str())).ok()?;
        Some(format!("{chunk_id}: {placeholder}"))
      })
      .collect::<Vec<_>>()
      .join(",")
  )
}

#[plugin_hook(RuntimePluginCreateScript for SubresourceIntegrityPlugin)]
pub async fn create_script(&self, mut data: CreateScriptData) -> Result<CreateScriptData> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.chunk.compilation_id);
  data.code = add_attribute(
    "script",
    &get_hash_varaiable(&ctx.runtime_template, SourceType::JavaScript),
    &data.code,
    &ctx.cross_origin_loading,
  );
  Ok(data)
}

#[plugin_hook(RuntimePluginCreateLink for SubresourceIntegrityPlugin)]
pub async fn create_link(&self, mut data: CreateLinkData) -> Result<CreateLinkData> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.chunk.compilation_id);
  if data.code.contains("loadingAttribute") {
    data.code = add_attribute(
      "link",
      &get_hash_varaiable(&ctx.runtime_template, SourceType::Css),
      &data.code,
      &ctx.cross_origin_loading,
    );
  } else {
    data.code = add_attribute(
      "linkTag",
      &get_hash_varaiable(
        &ctx.runtime_template,
        SourceType::Custom("css/mini-extract".into()),
      ),
      &data.code,
      &ctx.cross_origin_loading,
    );
  }

  Ok(data)
}

#[plugin_hook(RuntimePluginLinkPreload for SubresourceIntegrityPlugin)]
pub async fn link_preload(&self, mut data: LinkPreloadData) -> Result<LinkPreloadData> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.chunk.compilation_id);
  if data.code.contains(".as = \"style\"") {
    data.code = add_attribute(
      "link",
      (if data.code.contains(".miniCssF") {
        get_hash_varaiable(
          &ctx.runtime_template,
          SourceType::Custom("css/mini-extract".into()),
        )
      } else {
        get_hash_varaiable(&ctx.runtime_template, SourceType::Css)
      })
      .as_str(),
      &data.code,
      &ctx.cross_origin_loading,
    );
  } else {
    data.code = add_attribute(
      "link",
      &get_hash_varaiable(&ctx.runtime_template, SourceType::JavaScript),
      &data.code,
      &ctx.cross_origin_loading,
    );
  }

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
