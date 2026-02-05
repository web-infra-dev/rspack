use std::iter;

use itertools::Itertools;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupChunkDependenciesRuntimeModule {
  async_chunk_loading: bool,
}

impl StartupChunkDependenciesRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, async_chunk_loading: bool) -> Self {
    Self::with_default(runtime_template, async_chunk_loading)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/startup_chunk_dependencies.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk_ids = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_entry_dependent_chunks_iterable(
          &chunk_ukey,
          &compilation.build_chunk_graph_artifact.chunk_by_ukey,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        )
        .map(|chunk_ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id()
            .to_string()
        })
        .collect::<Vec<_>>();

      let body = if self.async_chunk_loading {
        match chunk_ids.len() {
          1 => format!(
            r#"return {}("{}").then(next);"#,
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
            chunk_ids.first().expect("Should has at least one chunk")
          ),
          2 => format!(
            r#"return Promise.all([{}]).then(next);"#,
            chunk_ids
              .iter()
              .map(|cid| format!(
                r#"{}("{}")"#,
                compilation
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
                cid
              ))
              .join(",\n")
          ),
          _ => format!(
            r#"return Promise.all({}.map({}, {})).then(next);"#,
            serde_json::to_string(&chunk_ids).expect("Invalid json to string"),
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::REQUIRE)
          ),
        }
      } else {
        chunk_ids
          .iter()
          .map(|cid| {
            format!(
              r#"{}("{}");"#,
              compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
              cid
            )
          })
          .chain(iter::once("return next();".to_string()))
          .join("\n")
      };

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_body": body,
        })),
      )?;

      Ok(source)
    } else {
      unreachable!("should have chunk for StartupChunkDependenciesRuntimeModule")
    }
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::STARTUP
      | RuntimeGlobals::ENSURE_CHUNK
      | RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES
  }
}
