use std::iter;

use itertools::Itertools;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupChunkDependenciesRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
  chunk: Option<ChunkUkey>,
}

impl StartupChunkDependenciesRuntimeModule {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/startup_chunk_dependencies"),
      async_chunk_loading,
      None,
    )
  }
}

impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/startup_chunk_dependencies.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk_ids = compilation
        .chunk_graph
        .get_chunk_entry_dependent_chunks_iterable(
          &chunk_ukey,
          &compilation.chunk_by_ukey,
          &compilation.chunk_group_by_ukey,
        )
        .map(|chunk_ukey| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id(&compilation.chunk_ids_artifact)
            .to_string()
        })
        .collect::<Vec<_>>();

      let body = if self.async_chunk_loading {
        match chunk_ids.len() {
          1 => format!(
            r#"return {}("{}").then(next);"#,
            RuntimeGlobals::ENSURE_CHUNK,
            chunk_ids.first().expect("Should has at least one chunk")
          ),
          2 => format!(
            r#"return Promise.all([{}]).then(next);"#,
            chunk_ids
              .iter()
              .map(|cid| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, cid))
              .join(",\n")
          ),
          _ => format!(
            r#"return Promise.all({}.map({}, {})).then(next);"#,
            serde_json::to_string(&chunk_ids).expect("Invalid json to string"),
            RuntimeGlobals::ENSURE_CHUNK,
            RuntimeGlobals::REQUIRE
          ),
        }
      } else {
        chunk_ids
          .iter()
          .map(|cid| format!(r#"{}("{}");"#, RuntimeGlobals::ENSURE_CHUNK, cid))
          .chain(iter::once("return next();".to_string()))
          .join("\n")
      };

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_body": body,
        })),
      )?;

      Ok(RawStringSource::from(source).boxed())
    } else {
      unreachable!("should have chunk for StartupChunkDependenciesRuntimeModule")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
