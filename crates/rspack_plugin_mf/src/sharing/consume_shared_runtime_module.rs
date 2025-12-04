use rspack_collections::Identifier;
use rspack_core::{
  Chunk, ChunkGraph, ChunkUkey, Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType, impl_runtime_module,
};
use rustc_hash::FxHashMap;

use super::consume_shared_plugin::ConsumeVersion;
use crate::utils::json_stringify;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ConsumeSharedRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  enhanced: bool,
}

impl ConsumeSharedRuntimeModule {
  pub fn new(enhanced: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/consumes_loading"),
      None,
      enhanced,
    )
  }

  fn get_template_id(&self, template_id: TemplateId) -> String {
    match template_id {
      TemplateId::Common => format!("{}_consumesCommon", self.id),
      TemplateId::Initial => format!("{}_consumesInitial", self.id),
      TemplateId::Loading => format!("{}_consumesLoading", self.id),
    }
  }
}

enum TemplateId {
  Common,
  Initial,
  Loading,
}

#[async_trait::async_trait]
impl RuntimeModule for ConsumeSharedRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.get_template_id(TemplateId::Common),
        include_str!("./consumesCommon.ejs").to_string(),
      ),
      (
        self.get_template_id(TemplateId::Initial),
        include_str!("./consumesInitial.ejs").to_string(),
      ),
      (
        self.get_template_id(TemplateId::Loading),
        include_str!("./consumesLoading.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let module_graph = compilation.get_module_graph();
    let mut chunk_to_module_mapping = FxHashMap::default();
    let mut module_id_to_consume_data_mapping = FxHashMap::default();
    let mut initial_consumes = Vec::new();
    let mut add_module = |module: ModuleIdentifier, chunk: &Chunk, ids: &mut Vec<String>| {
      let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)
        .map(|s| s.to_string())
        .expect("should have moduleId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
      ids.push(id.clone());
      let code_gen = compilation
        .code_generation_results
        .get(&module, Some(chunk.runtime()));
      if let Some(data) = code_gen.data.get::<CodeGenerationDataConsumeShared>() {
        module_id_to_consume_data_mapping.insert(id, format!(
          "{{ shareScope: {}, shareKey: {}, import: {}, requiredVersion: {}, strictVersion: {}, singleton: {}, eager: {}, fallback: {} }}",
          json_stringify(&data.share_scope),
          json_stringify(&data.share_key),
          json_stringify(&data.import),
          json_stringify(&data.required_version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "*".to_string())),
          json_stringify(&data.strict_version),
          json_stringify(&data.singleton),
          json_stringify(&data.eager),
          data.fallback.as_deref().unwrap_or("undefined"),
        ));
      }
    };
    for chunk in chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          &module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      let mut ids = vec![];
      for mid in modules {
        add_module(mid, chunk, &mut ids);
      }
      if ids.is_empty() {
        continue;
      }
      chunk_to_module_mapping.insert(
        chunk
          .id()
          .map(ToOwned::to_owned)
          .expect("should have chunkId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate"),
        ids,
      );
    }
    for chunk in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          &module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      for mid in modules {
        add_module(mid, chunk, &mut initial_consumes);
      }
    }
    let module_id_to_consume_data_mapping = if module_id_to_consume_data_mapping.is_empty() {
      "{}".to_string()
    } else {
      format!(
        "{{{}}}",
        module_id_to_consume_data_mapping
          .into_iter()
          .map(|(k, v)| format!("{}: {}", json_stringify(&k), v))
          .collect::<Vec<_>>()
          .join(", ")
      )
    };
    let chunk_mapping = if chunk_to_module_mapping.is_empty() {
      "{}".to_string()
    } else {
      json_stringify(&chunk_to_module_mapping)
    };
    let initial_consumes_json = if initial_consumes.is_empty() {
      "[]".to_string()
    } else {
      json_stringify(&initial_consumes)
    };
    let require_name = compilation
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let mut source = format!(
      r#"
{require_name}.consumesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToConsumeDataMapping: {module_to_consume_data_mapping}, initialConsumes: {initial_consumes_json} }};
"#,
      require_name = require_name,
      chunk_mapping = chunk_mapping,
      module_to_consume_data_mapping = module_id_to_consume_data_mapping,
      initial_consumes_json = initial_consumes_json,
    );
    if self.enhanced {
      if ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey)
        .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      {
        source += &format!(
          "{ensure_chunk_handlers}.consumes = {ensure_chunk_handlers}.consumes || function() {{ throw new Error(\"should have {ensure_chunk_handlers}.consumes\") }}",
          ensure_chunk_handlers = compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        );
      }
      return Ok(source);
    }
    source += &compilation
      .runtime_template
      .render(&self.get_template_id(TemplateId::Common), None)?;
    if !initial_consumes.is_empty() {
      source += &compilation
        .runtime_template
        .render(&self.get_template_id(TemplateId::Initial), None)?;
    }
    if ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey)
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      source += &compilation
        .runtime_template
        .render(&self.get_template_id(TemplateId::Loading), None)?;
    }
    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataConsumeShared {
  pub share_scope: String,
  pub share_key: String,
  pub import: Option<String>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
  pub fallback: Option<String>,
}
