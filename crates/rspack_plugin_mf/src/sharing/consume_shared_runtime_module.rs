use std::{collections::BTreeMap, sync::LazyLock};

use rspack_core::{
  Chunk, ChunkGraph, Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeModuleRuntimeRequirements, RuntimeModuleStage,
  RuntimeTemplate, SourceType, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_globals_dependencies_from_ejs;
use rspack_util::json_stringify_str;

use super::consume_shared_plugin::ConsumeVersion;
use crate::{
  ShareScope,
  utils::{json_stringify, runtime_require_scope_name, runtime_require_scope_requirement},
};

static CONSUMES_COMMON_TEMPLATE: &str = include_str!("./consumesCommon.ejs");
static CONSUMES_INITIAL_TEMPLATE: &str = include_str!("./consumesInitial.ejs");
static CONSUMES_LOADING_TEMPLATE: &str = include_str!("./consumesLoading.ejs");
static CONSUMES_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| RuntimeModuleRuntimeRequirements {
    dependencies: extract_runtime_globals_dependencies_from_ejs(
      CONSUMES_COMMON_TEMPLATE,
      RuntimeGlobals::default(),
    ),
    ..Default::default()
  });
static CONSUMES_INITIAL_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| RuntimeModuleRuntimeRequirements {
    dependencies: extract_runtime_globals_dependencies_from_ejs(
      CONSUMES_INITIAL_TEMPLATE,
      RuntimeGlobals::default(),
    ),
    ..Default::default()
  });
static CONSUMES_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| RuntimeModuleRuntimeRequirements {
    dependencies: extract_runtime_globals_dependencies_from_ejs(
      CONSUMES_LOADING_TEMPLATE,
      RuntimeGlobals::default(),
    ),
    ..Default::default()
  });

#[impl_runtime_module]
#[derive(Debug)]
pub struct ConsumeSharedRuntimeModule {
  enhanced: bool,
}

impl ConsumeSharedRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, enhanced: bool) -> Self {
    Self::with_name(runtime_template, "consumes_loading", enhanced)
  }

  fn get_template_id(&self, template_id: TemplateId) -> String {
    match template_id {
      TemplateId::Common => format!("{}_consumesCommon", self.id()),
      TemplateId::Initial => format!("{}_consumesInitial", self.id()),
      TemplateId::Loading => format!("{}_consumesLoading", self.id()),
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
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let dependencies = CONSUMES_RUNTIME_REQUIREMENTS.dependencies
      | CONSUMES_INITIAL_RUNTIME_REQUIREMENTS.dependencies
      | CONSUMES_LOADING_RUNTIME_REQUIREMENTS.dependencies
      | runtime_require_scope_requirement(compilation);
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      ..Default::default()
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.get_template_id(TemplateId::Common),
        CONSUMES_COMMON_TEMPLATE.to_string(),
      ),
      (
        self.get_template_id(TemplateId::Initial),
        CONSUMES_INITIAL_TEMPLATE.to_string(),
      ),
      (
        self.get_template_id(TemplateId::Loading),
        CONSUMES_LOADING_TEMPLATE.to_string(),
      ),
    ]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk_ukey = self
      .chunk()
      .expect("should have chunk in <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    let module_graph = compilation.get_module_graph();
    let mut chunk_to_module_mapping = BTreeMap::default();
    let mut module_id_to_consume_data_mapping = BTreeMap::default();
    let mut initial_consumes = Vec::new();
    let enhanced = self.enhanced;
    let mut add_module = |module: ModuleIdentifier, chunk: &Chunk, ids: &mut Vec<String>| {
      let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)
        .map(|s| s.to_string())
        .expect("should have moduleId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
      ids.push(id.clone());
      let code_gen = compilation
        .code_generation_results
        .get(&module, Some(chunk.runtime()));
      if let Some(data) = code_gen.data.get::<CodeGenerationDataConsumeShared>() {
        let share_scope_json = if enhanced {
          json_stringify(&data.share_scope)
        } else {
          json_stringify_str(
            data
              .share_scope
              .scopes()
              .first()
              .map_or("default", |s| s.as_str()),
          )
        };
        module_id_to_consume_data_mapping.insert(id, format!(
          "{{ shareScope: {}, shareKey: {}, import: {}, requiredVersion: {}, strictVersion: {}, singleton: {}, eager: {}, fallback: {}, treeShakingMode: {} }}",
          share_scope_json,
          json_stringify(&data.share_key),
          json_stringify(&data.import),
          json_stringify_str(&data.required_version.as_ref().map_or_else(|| "*".to_string(), |v| v.to_string())),
          json_stringify(&data.strict_version),
          json_stringify(&data.singleton),
          json_stringify(&data.eager),
          data.fallback.as_deref().unwrap_or("undefined"),
          json_stringify(&data.tree_shaking_mode),
        ));
      }
    };
    // Match enhanced/webpack behavior: include all referenced chunks so async ones are mapped too
    for chunk in
      chunk.get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      let mut modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          module_graph,
        );
      modules.sort_unstable();
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk);
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
          .map(|id| id.to_string())
          .expect("should have chunkId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate"),
        ids,
      );
    }
    for chunk in
      chunk.get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      let mut modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          module_graph,
        );
      modules.sort_unstable();
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk);
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
          .map(|(k, v)| format!("{}: {}", json_stringify_str(&k), v))
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
    let require_name = runtime_require_scope_name(runtime_template);
    let mut source = format!(
      r#"
{require_name}.consumesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToConsumeDataMapping: {module_id_to_consume_data_mapping}, initialConsumes: {initial_consumes_json} }};
"#,
    );
    if self.enhanced {
      if ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey)
        .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      {
        source += &format!(
          "{ensure_chunk_handlers}.consumes = {ensure_chunk_handlers}.consumes || function() {{ throw new Error(\"should have {ensure_chunk_handlers}.consumes\") }}",
          ensure_chunk_handlers =
            runtime_template.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        );
      }
      return Ok(source);
    }
    source += &runtime_template.render(&self.get_template_id(TemplateId::Common), None)?;
    if !initial_consumes.is_empty() {
      source += &runtime_template.render(&self.get_template_id(TemplateId::Initial), None)?;
    }
    if ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey)
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      source += &runtime_template.render(&self.get_template_id(TemplateId::Loading), None)?;
    }
    Ok(source)
  }
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataConsumeShared {
  pub share_scope: ShareScope,
  pub share_key: String,
  pub import: Option<String>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
  pub fallback: Option<String>,
  pub tree_shaking_mode: Option<String>,
}
