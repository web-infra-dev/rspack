use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirements, CrossOriginLoading, ManifestAssetType,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt, RuntimeModuleGenerateContext, RuntimeTemplate,
  SourceType, chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rspack_error::{Result, error};
use rspack_hook::plugin_hook;
use rspack_plugin_runtime::{
  CreateLinkData, CreateScriptData, LinkPreloadData, RuntimePluginCreateLink,
  RuntimePluginCreateScript, RuntimePluginLinkPreload,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  IntegrityHashLoading, SubresourceIntegrityHashFunction, SubresourceIntegrityPlugin,
  SubresourceIntegrityPluginInner,
  util::{find_chunks, get_hash_variable, make_placeholder},
};

fn add_attribute(
  tag: &str,
  variable_ref: &str,
  code: &str,
  cross_origin_loading: &CrossOriginLoading,
) -> String {
  format!(
    r#"{code}
{tag}.integrity = {variable_ref}[chunkId];
{tag}.crossOrigin = '{cross_origin_loading}';"#
  )
}

#[impl_runtime_module]
#[derive(Debug)]
struct SRIHashVariableRuntimeModule {
  hash_funcs: Vec<SubresourceIntegrityHashFunction>,
  /// Set for `hashLoading: "lazy"`: the runtime chunk then carries only the JS
  /// hashes in its own manifest, and other chunks add theirs as they load.
  lazy_manifest: Option<Vec<ChunkUkey>>,
}

impl SRIHashVariableRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    hash_funcs: Vec<SubresourceIntegrityHashFunction>,
    lazy_manifest: Option<Vec<ChunkUkey>>,
  ) -> Self {
    Self::with_default(runtime_template, hash_funcs, lazy_manifest)
  }
}

/// Whether a chunk renders anything that integrity needs to cover.
fn chunk_has_code(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
  let module_graph = compilation.get_module_graph();
  let extract_css_source_type = SourceType::Custom("css/mini-extract".into());
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules(chunk, module_graph)
    .iter()
    .any(|m| {
      // `css/mini-extract` modules deliberately generate no code: their content is
      // emitted as a standalone css asset. A chunk holding them is not empty, so it
      // must not be judged by its code generation result.
      if m
        .source_types(module_graph)
        .contains(&extract_css_source_type)
      {
        return true;
      }
      let result = compilation.code_generation_results.get_one(&m.identifier());
      result.sources().values().any(|v| v.size() != 0)
    })
}

#[async_trait::async_trait]
impl RuntimeModule for SRIHashVariableRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    let compilation = context.compilation;
    let Some(chunk) = self
      .chunk()
      .as_ref()
      .and_then(|c| compilation.build_chunk_graph_artifact.chunk_by_ukey.get(c))
    else {
      return Err(error!(
        "Generate sri runtime module failed: chunk not found"
      ));
    };

    let include_chunks = chunk
      .get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      .iter()
      .filter_map(|c| {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(c)?;
        let id = chunk.id()?;
        let rendered_hash = chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        )?;
        Some((id, rendered_hash))
      })
      .collect::<HashMap<_, _>>();

    let module_graph = compilation.get_module_graph();

    let runtime_template = context.runtime_template;
    let runtime_require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let extract_css_source_type = SourceType::Custom("css/mini-extract".into());
    let source_types = vec![
      (
        SourceType::JavaScript,
        get_hash_variable(&runtime_require_name, SourceType::JavaScript),
      ),
      (
        SourceType::Css,
        get_hash_variable(&runtime_require_name, SourceType::Css),
      ),
      (
        extract_css_source_type,
        get_hash_variable(&runtime_require_name, extract_css_source_type),
      ),
    ];

    let runtime_chunk = self.chunk().expect("should attached chunk");
    let all_chunks = find_chunks(&runtime_chunk, compilation)
      .into_iter()
      .filter(|c| chunk_has_code(c, compilation))
      .collect::<Vec<_>>();

    let mut code = vec![];

    for (source_type, variable_ref) in source_types {
      let mut chunk_with_source_type = all_chunks
        .iter()
        .filter(|c| {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .has_chunk_module_by_source_type(c, source_type, module_graph)
        })
        .map(|c| {
          (
            *c,
            compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .expect_get(c)
              .expect_id(),
          )
        })
        .filter(|(_, id)| include_chunks.contains_key(id))
        .collect::<Vec<_>>();

      // Lazy loading leaves the variable in place, even when empty, so the chunks
      // loaded later can add their children's hashes to it.
      if !chunk_with_source_type.is_empty() {
        if let Some(lazy_manifest) = &self.lazy_manifest
          && matches!(source_type, SourceType::JavaScript)
        {
          chunk_with_source_type.retain(|(c, _)| lazy_manifest.contains(c));
        }
        let chunk_with_source_type = chunk_with_source_type
          .into_iter()
          .map(|(_, id)| id)
          .collect::<Vec<_>>();
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
          ),
        ));
      }
    }

    Ok(code.join("\n"))
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { RuntimeGlobals::REQUIRE_SCOPE },
      ..Default::default()
    }
  }
}

fn generate_sri_hash_placeholders(
  asset_type: ManifestAssetType,
  chunks: Vec<&ChunkId>,
  hash_funcs: &Vec<SubresourceIntegrityHashFunction>,
) -> String {
  format!(
    "{{{}}}",
    chunks
      .into_iter()
      .map(|c| {
        let chunk_id = rspack_util::json_stringify(c);
        let placeholder =
          rspack_util::json_stringify_str(&make_placeholder(asset_type, hash_funcs, c.as_str()));
        format!("{chunk_id}: {placeholder}")
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
    &get_hash_variable(&ctx.runtime_require_name, SourceType::JavaScript),
    &data.code,
    &ctx.cross_origin_loading,
  );
  Ok(data)
}

#[plugin_hook(RuntimePluginCreateLink for SubresourceIntegrityPlugin)]
pub async fn create_link<'a>(
  &self,
  compilation: &Compilation,
  mut data: CreateLinkData<'a>,
) -> Result<CreateLinkData<'a>> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(compilation.id());
  let runtime_template = compilation
    .runtime_template
    .create_runtime_module_code_template();
  let runtime_require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
  if data.code.contains("loadingAttribute") {
    data.code = add_attribute(
      "link",
      &get_hash_variable(&runtime_require_name, SourceType::Css),
      &data.code,
      &ctx.cross_origin_loading,
    );
  } else {
    data.code = add_attribute(
      "linkTag",
      &get_hash_variable(
        &runtime_require_name,
        SourceType::Custom("css/mini-extract".into()),
      ),
      &data.code,
      &ctx.cross_origin_loading,
    );
  }

  Ok(data)
}

#[plugin_hook(RuntimePluginLinkPreload for SubresourceIntegrityPlugin)]
pub async fn link_preload<'a>(
  &self,
  compilation: &Compilation,
  mut data: LinkPreloadData<'a>,
) -> Result<LinkPreloadData<'a>> {
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(compilation.id());
  let runtime_template = compilation
    .runtime_template
    .create_runtime_module_code_template();
  let runtime_require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
  if data.code.contains(".as = \"style\"") {
    data.code = add_attribute(
      "link",
      (if data.code.contains(".miniCssF") {
        get_hash_variable(
          &runtime_require_name,
          SourceType::Custom("css/mini-extract".into()),
        )
      } else {
        get_hash_variable(&runtime_require_name, SourceType::Css)
      })
      .as_str(),
      &data.code,
      &ctx.cross_origin_loading,
    );
  } else {
    data.code = add_attribute(
      "link",
      &get_hash_variable(&runtime_require_name, SourceType::JavaScript),
      &data.code,
      &ctx.cross_origin_loading,
    );
  }

  Ok(data)
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for SubresourceIntegrityPlugin)]
pub async fn handle_runtime(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let lazy_manifest = matches!(self.options.hash_loading, IntegrityHashLoading::Lazy).then(|| {
    SubresourceIntegrityPlugin::get_compilation_manifest(compilation)
      .manifest
      .get(chunk_ukey)
      .map(|children| children.iter().copied().collect())
      .unwrap_or_default()
  });
  runtime_modules.push(
    SRIHashVariableRuntimeModule::new(
      &compilation.runtime_template,
      self.options.hash_func_names.clone(),
      lazy_manifest,
    )
    .boxed(),
  );
  Ok(())
}

/// Adds the hashes of a chunk's direct async children when the chunk loads, for
/// `hashLoading: "lazy"`.
#[impl_runtime_module]
#[derive(Debug)]
struct SRILazyHashesRuntimeModule {
  hash_funcs: Vec<SubresourceIntegrityHashFunction>,
  child_chunks: Vec<ChunkUkey>,
}

impl SRILazyHashesRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    chunk_id: &ChunkId,
    hash_funcs: Vec<SubresourceIntegrityHashFunction>,
    child_chunks: Vec<ChunkUkey>,
  ) -> Self {
    // Runtime modules are keyed by runtime and name, and chunks sharing a runtime
    // each carry their own hashes, so the name includes the chunk.
    Self::with_name(
      runtime_template,
      &format!("sri_lazy_hashes_{chunk_id}"),
      hash_funcs,
      child_chunks,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for SRILazyHashesRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    let compilation = context.compilation;
    let module_graph = compilation.get_module_graph();
    let mut chunk_ids = self
      .child_chunks
      .iter()
      .filter(|c| {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .has_chunk_module_by_source_type(c, SourceType::JavaScript, module_graph)
          && chunk_has_code(c, compilation)
      })
      .filter_map(|c| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(c)
          .id()
      })
      .collect::<Vec<_>>();
    if chunk_ids.is_empty() {
      return Ok(String::new());
    }
    chunk_ids.sort();

    Ok(format!(
      "Object.assign({}, {});",
      get_hash_variable(
        // The module renders inside a chunk, where require is reached through the
        // chunk's runtime argument rather than the runtime chunk's own binding.
        &compilation
          .runtime_template
          .create_chunk_code_template()
          .render_runtime_globals(&RuntimeGlobals::REQUIRE),
        SourceType::JavaScript
      ),
      generate_sri_hash_placeholders(ManifestAssetType::JavaScript, chunk_ids, &self.hash_funcs),
    ))
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { RuntimeGlobals::REQUIRE_SCOPE },
      ..Default::default()
    }
  }
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for SubresourceIntegrityPlugin)]
pub async fn handle_chunk_runtime(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  // A runtime chunk carries its manifest in the hash variable itself.
  if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    return Ok(());
  }
  let Some(chunk_id) = chunk.id() else {
    return Ok(());
  };
  let manifest = SubresourceIntegrityPlugin::get_compilation_manifest(compilation);
  let Some(child_chunks) = manifest.manifest.get(chunk_ukey) else {
    return Ok(());
  };
  if child_chunks.is_empty() {
    return Ok(());
  }
  runtime_modules.push(
    SRILazyHashesRuntimeModule::new(
      &compilation.runtime_template,
      chunk_id,
      self.options.hash_func_names.clone(),
      child_chunks.iter().copied().collect(),
    )
    .boxed(),
  );
  Ok(())
}
