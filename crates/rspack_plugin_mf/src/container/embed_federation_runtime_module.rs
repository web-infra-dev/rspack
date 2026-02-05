//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

use rspack_cacheable::cacheable;
use rspack_core::{
  Compilation, DependencyId, RuntimeModule, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::Result;

use super::module_federation_runtime_plugin::ModuleFederationRuntimeExperimentsOptions;

#[cacheable]
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EmbedFederationRuntimeModuleOptions {
  pub collected_dependency_ids: Vec<DependencyId>,
  pub experiments: ModuleFederationRuntimeExperimentsOptions,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct EmbedFederationRuntimeModule {
  options: EmbedFederationRuntimeModuleOptions,
}

impl EmbedFederationRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    options: EmbedFederationRuntimeModuleOptions,
  ) -> Self {
    Self::with_name(runtime_template, "embed_federation_runtime", options)
  }
}

enum TemplateId {
  Async,
  Sync,
}

impl EmbedFederationRuntimeModule {
  fn template_id(&self, template_id: TemplateId) -> String {
    match template_id {
      TemplateId::Async => format!("{}_async", self.id),
      TemplateId::Sync => format!("{}_sync", self.id),
    }
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EmbedFederationRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Async),
        include_str!("./embed_federation_runtime_async.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::Sync),
        include_str!("./embed_federation_runtime_sync.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    let chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let collected_deps = &self.options.collected_dependency_ids;

    let module_graph = compilation.get_module_graph();
    let mut federation_runtime_modules = Vec::new();

    // Find federation runtime dependencies in this chunk
    if !collected_deps.is_empty() {
      for dep_id in collected_deps.iter() {
        if let Some(module_dyn) = module_graph.get_module_by_dependency_id(dep_id) {
          let is_in_chunk = compilation
            .chunk_graph
            .is_module_in_chunk(&module_dyn.identifier(), chunk_ukey);
          if is_in_chunk {
            federation_runtime_modules.push(*dep_id);
          }
        }
      }
    }

    // Generate module execution code for each federation runtime dependency
    let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 64);
    let mut runtime_template = compilation
      .runtime_template
      .create_module_codegen_runtime_template();

    for dep_id in federation_runtime_modules {
      let module_str = runtime_template.module_raw(compilation, &dep_id, "", false);
      module_executions.push_str("\t\t");
      module_executions.push_str(&module_str);
      module_executions.push('\n');
    }

    if self.options.experiments.async_startup {
      let entry_chunk_ids = compilation
        .chunk_by_ukey
        .expect_get(&chunk_ukey)
        .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id()
            .to_string()
        })
        .collect::<Vec<_>>();
      let entry_chunk_ids_literal =
        serde_json::to_string(&entry_chunk_ids).expect("Invalid json to string");
      Ok(compilation.runtime_template.render(
        &self.template_id(TemplateId::Async),
        Some(serde_json::json!({
          "_module_executions": module_executions,
          "_entry_chunk_ids": entry_chunk_ids_literal,
        })),
      )?)
    } else {
      if module_executions.is_empty() {
        return Ok("// Federation runtime entry modules not found in this chunk.".into());
      }
      // Sync startup: keep the legacy prevStartup wrapper for minimal surface area.
      Ok(compilation.runtime_template.render(
        &self.template_id(TemplateId::Sync),
        Some(serde_json::json!({
          "_module_executions": module_executions,
        })),
      )?)
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }
}
