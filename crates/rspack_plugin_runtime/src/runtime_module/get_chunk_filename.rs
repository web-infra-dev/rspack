use std::{cmp::Ordering, fmt};

use itertools::Itertools;
use rspack_cacheable::with::Unsupported;
use rspack_core::{
  Chunk, ChunkGraph, ChunkUkey, Compilation, Filename, PathData, RuntimeGlobals,
  RuntimeGlobalsRenderMode, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  SourceType, impl_runtime_module,
};
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa,
};
use rustc_hash::FxHashMap;

use super::{stringify_dynamic_chunk_map, stringify_static_chunk_map};
use crate::{get_chunk_runtime_requirements, runtime_module::unquoted_stringify};

type GetChunkFilenameAllChunks = Box<dyn Fn(&RuntimeGlobals) -> bool + Sync + Send>;
type GetFilenameForChunk = Box<dyn Fn(&Chunk, &Compilation) -> Option<Filename> + Sync + Send>;

#[impl_runtime_module]
pub struct GetChunkFilenameRuntimeModule {
  #[cacheable(with=Unsupported)]
  content_type: &'static str,
  source_type: SourceType,
  global: String,
  rspack_export_global: Option<String>,
  #[cacheable(with=Unsupported)]
  all_chunks: GetChunkFilenameAllChunks,
  #[cacheable(with=Unsupported)]
  filename_for_chunk: GetFilenameForChunk,
  chunk_ukey: ChunkUkey,
}

impl fmt::Debug for GetChunkFilenameRuntimeModule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GetChunkFilenameRuntimeModule")
      .field("id", self.id())
      .field("chunk", &self.chunk())
      .field("content_type", &self.content_type)
      .field("source_type", &self.source_type)
      .field("global", &self.global)
      .field("all_chunks", &"...")
      .field("chunk_ukey", &self.chunk_ukey)
      .finish()
  }
}

// It's render is different with webpack, rspack will only render chunk map<chunkId, chunkName>
// and search it.
impl GetChunkFilenameRuntimeModule {
  pub fn new<
    F: Fn(&RuntimeGlobals) -> bool + Sync + Send + 'static,
    T: Fn(&Chunk, &Compilation) -> Option<Filename> + Sync + Send + 'static,
  >(
    runtime_template: &RuntimeTemplate,
    kind: (&'static str, &'static str),
    source_type: SourceType,
    global: String,
    all_chunks: F,
    filename_for_chunk: T,
    chunk_ukey: ChunkUkey,
  ) -> Self {
    let (content_type, name) = kind;
    Self::with_name(
      runtime_template,
      &format!("get {name} chunk filename"),
      content_type,
      source_type,
      global,
      None,
      Box::new(all_chunks),
      Box::new(filename_for_chunk),
      chunk_ukey,
    )
  }

  pub fn with_rspack_export_global(mut self, global: impl Into<String>) -> Self {
    self.rspack_export_global = Some(global.into());
    self
  }

  fn get_filename_chunks(&self, compilation: &Compilation) -> Option<FxIndexSet<ChunkUkey>> {
    let chunk_ukey = self.chunk().unwrap_or(self.chunk_ukey);
    compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(&chunk_ukey)
      .map(|chunk| {
        let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());
        let mut chunks = if (self.all_chunks)(runtime_requirements) {
          chunk
            .get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
        } else {
          let mut chunks =
            chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);

          if ChunkGraph::get_tree_runtime_requirements(compilation, &chunk.ukey())
            .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
          {
            chunks.extend(
              compilation
                .build_chunk_graph_artifact
                .chunk_graph
                .get_runtime_chunk_dependent_chunks_iterable(
                  &chunk.ukey(),
                  &compilation.build_chunk_graph_artifact.chunk_by_ukey,
                  &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
                ),
            );
          }
          chunks
        };
        for entrypoint in chunk.get_all_referenced_async_entrypoints(
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        ) {
          let entrypoint = compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .expect_get(&entrypoint);
          chunks.insert(entrypoint.get_entrypoint_chunk());
        }
        chunks
      })
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: {
        if (self.source_type == SourceType::JavaScript
          && compilation
            .options
            .output
            .chunk_filename
            .has_hash_placeholder())
          || (self.source_type == SourceType::Css
            && compilation
              .options
              .output
              .css_chunk_filename
              .has_hash_placeholder())
        {
          RuntimeGlobals::GET_FULL_HASH
        } else {
          RuntimeGlobals::default()
        }
      },
      define: {
        match self.source_type {
          SourceType::JavaScript => RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME,
          SourceType::Css => RuntimeGlobals::GET_CHUNK_CSS_FILENAME,
          _ => RuntimeGlobals::default(),
        }
      },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/get_chunk_filename.ejs").to_string(),
    )]
  }

  fn dependent_hash(&self) -> bool {
    true
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunks = self.get_filename_chunks(compilation);

    let mut dynamic_filename: Option<String> = None;
    let mut max_chunk_set_size = 0;
    let mut chunk_filenames = Vec::<(Filename, ChunkUkey)>::new();
    let mut chunk_set_sizes_by_filenames = FxHashMap::<String, usize>::default();
    let mut chunk_map = FxIndexMap::default();

    if let Some(chunks) = chunks {
      chunks
        .iter()
        .filter_map(|chunk_ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .get(chunk_ukey)
        })
        .for_each(|chunk| {
          let filename = (self.filename_for_chunk)(chunk, compilation);

          if let Some(filename) = filename {
            chunk_map.insert(chunk.ukey(), chunk);

            chunk_filenames.push((filename.clone(), chunk.ukey()));

            if let Some(filename_template) = filename.template() {
              let chunk_set_size = chunk_set_sizes_by_filenames
                .entry(filename_template.to_owned())
                .or_insert(0);
              *chunk_set_size += 1;
              let chunk_set_size = *chunk_set_size;
              let should_update = match dynamic_filename {
                Some(ref dynamic_filename) => match chunk_set_size.cmp(&max_chunk_set_size) {
                  Ordering::Less => false,
                  Ordering::Greater => true,
                  Ordering::Equal => match filename_template.len().cmp(&dynamic_filename.len()) {
                    Ordering::Less => false,
                    Ordering::Greater => true,
                    Ordering::Equal => !matches!(
                      filename_template.cmp(dynamic_filename.as_str()),
                      Ordering::Less
                    ),
                  },
                },
                None => true,
              };
              if should_update {
                max_chunk_set_size = chunk_set_size;
                dynamic_filename = Some(filename_template.to_owned());
              }
            };
          }
        });
    }

    let dynamic_url = if let Some(dynamic_filename) = &dynamic_filename {
      let chunks = chunk_filenames
        .iter()
        .filter_map(|(filename, chunk)| {
          if filename.template() == Some(dynamic_filename.as_str()) {
            Some(*chunk)
          } else {
            None
          }
        })
        .collect::<FxIndexSet<ChunkUkey>>();
      let filename = Filename::from(dynamic_filename.clone());
      let compiled = filename
        .compiled_template()
        .expect("dynamic filename is always a template");
      let fake_filename = Filename::from(compiled.without_hash_length());
      let chunk_hash_len = compiled.chunk_hash_len();
      let content_hash_len = compiled.content_hash_len();
      let full_hash_len = compiled.full_hash_len().or(compiled.hash_len());

      let chunk_id = "\" + chunkId + \"";
      let chunk_name = stringify_dynamic_chunk_map(
        |c| c.name_for_filename_template().map(|s| s.to_string()),
        &chunks,
        &chunk_map,
      );
      let chunk_runtime = stringify_dynamic_chunk_map(
        |c| {
          let runtime = c.runtime().as_str();
          Some(runtime.to_string())
        },
        &chunks,
        &chunk_map,
      );
      let chunk_hash = stringify_dynamic_chunk_map(
        |c| {
          let hash = c
            .rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            )
            .map(|hash| hash.to_string());
          match chunk_hash_len {
            Some(hash_len) => hash.map(|s| s[..hash_len].to_string()),
            None => hash,
          }
        },
        &chunks,
        &chunk_map,
      );
      let content_hash = stringify_dynamic_chunk_map(
        |c| {
          c.rendered_content_hash_by_source_type(
            &compilation.chunk_hashes_artifact,
            &self.source_type,
            compilation.options.output.hash_digest_length,
          )
          .map(|hash| match content_hash_len {
            Some(hash_len) => hash[..hash_len].to_string(),
            None => hash.to_string(),
          })
        },
        &chunks,
        &chunk_map,
      );
      let full_hash = match full_hash_len {
        Some(hash_len) => {
          let mut hash_len_buffer = itoa::Buffer::new();
          let hash_len_str = hash_len_buffer.format(hash_len);
          format!(
            "\" + {}().slice(0, {}) + \"",
            runtime_template.render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH),
            hash_len_str
          )
        }
        None => format!(
          "\" + {}() + \"",
          runtime_template.render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH)
        ),
      };

      Some(
        compilation
          .get_path(
            &Filename::from(rspack_util::json_stringify_str(fake_filename.as_str())),
            PathData::default()
              .chunk_id(chunk_id)
              .chunk_hash(&chunk_hash)
              .chunk_name(&chunk_name)
              .hash(&full_hash)
              .content_hash(&content_hash)
              .runtime(&chunk_runtime),
          )
          .await?,
      )
    } else {
      None
    };

    let mut static_urls = FxIndexMap::default();
    for (filename_template, chunk_ukey) in
      chunk_filenames
        .iter()
        .filter(|(filename, _)| match &dynamic_filename {
          None => true,
          Some(dynamic_filename) => filename.template() != Some(dynamic_filename.as_str()),
        })
    {
      if let Some(chunk) = chunk_map.get(chunk_ukey) {
        let compiled = filename_template
          .compiled(
            PathData::default()
              .chunk(chunk.ukey(), compilation)
              .chunk_name_optional(chunk.name())
              .chunk_id_optional(chunk.id().map(|id| id.as_str())),
            None,
          )
          .await?;
        let fake_filename = Filename::from(compiled.without_hash_length());
        let chunk_hash_len = compiled.chunk_hash_len();
        let content_hash_len = compiled.content_hash_len();
        let full_hash_len = compiled.full_hash_len().or(compiled.hash_len());

        let chunk_id = chunk
          .id()
          .map(|chunk_id| unquoted_stringify(Some(chunk_id), chunk_id.as_str()));
        let chunk_name = match chunk.name() {
          Some(chunk_name) => Some(unquoted_stringify(chunk.id(), chunk_name)),
          None => chunk
            .id()
            .map(|chunk_id| unquoted_stringify(Some(chunk_id), chunk_id.as_str())),
        };
        let chunk_hash = chunk
          .rendered_hash(
            &compilation.chunk_hashes_artifact,
            compilation.options.output.hash_digest_length,
          )
          .map(|chunk_hash| {
            let hash = unquoted_stringify(chunk.id(), chunk_hash);
            match chunk_hash_len {
              Some(hash_len) => hash[..hash_len].to_string(),
              None => hash,
            }
          });
        let content_hash = chunk
          .content_hash(&compilation.chunk_hashes_artifact)
          .and_then(|content_hash| content_hash.get(&self.source_type))
          .map(|i| {
            let hash = unquoted_stringify(
              chunk.id(),
              i.rendered(compilation.options.output.hash_digest_length),
            );
            match content_hash_len {
              Some(hash_len) => hash[..hash_len].to_string(),
              None => hash,
            }
          });
        let full_hash = match full_hash_len {
          Some(hash_len) => {
            let mut hash_len_buffer = itoa::Buffer::new();
            let hash_len_str = hash_len_buffer.format(hash_len);
            format!(
              "\" + {}().slice(0, {}) + \"",
              runtime_template.render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH),
              hash_len_str
            )
          }
          None => format!(
            "\" + {}() + \"",
            runtime_template.render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH)
          ),
        };
        let chunk_runtime = chunk.runtime().as_str();

        let filename = compilation
          .get_path(
            &Filename::from(rspack_util::json_stringify_str(fake_filename.as_str())),
            PathData::default()
              .chunk_id_optional(chunk_id.as_deref())
              .chunk_hash_optional(chunk_hash.as_deref())
              .chunk_name_optional(chunk_name.as_deref())
              .hash(&full_hash)
              .content_hash_optional(content_hash.as_deref())
              .runtime(chunk_runtime),
          )
          .await?;

        if let Some(chunk_id) = chunk.id() {
          static_urls
            .entry(filename)
            .or_insert(Vec::new())
            .push(chunk_id);
        }
      }
    }

    let custom_global = if runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackExport
    {
      self
        .rspack_export_global
        .as_ref()
        .map_or_else(|| self.global.clone(), |global| format!("var {global}"))
    } else {
      self.global.clone()
    };

    let source = runtime_template.render(self.id(), Some(serde_json::json!({
      "_global": match self.source_type {
        SourceType::JavaScript => runtime_template
          .render_runtime_global_definition(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        SourceType::Css => {
          runtime_template.render_runtime_global_definition(&RuntimeGlobals::GET_CHUNK_CSS_FILENAME)
        }
        _ => custom_global,
      },
      "_static_urls": static_urls
                        .iter()
                        .map(|(filename, chunk_ids)| stringify_static_chunk_map(filename, chunk_ids))
                        .join("\n"),
      "_dynamic_url": dynamic_url.unwrap_or_else(|| format!("\"\" + chunkId + \".{}\"", self.content_type))
    })))?;

    Ok(source)
  }
}
