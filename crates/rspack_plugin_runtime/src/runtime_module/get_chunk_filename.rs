use std::{cmp::Ordering, fmt};

use indexmap::IndexMap;
use itertools::Itertools;
use rspack_cacheable::with::Unsupported;
use rspack_collections::{DatabaseItem, Identifier, UkeyIndexMap, UkeyIndexSet};
use rspack_core::{
  get_filename_without_hash_length, impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Chunk, ChunkGraph, ChunkUkey, Compilation, Filename, FilenameTemplate, NoFilenameFn, PathData,
  RuntimeGlobals, RuntimeModule, SourceType,
};
use rspack_util::{infallible::ResultInfallibleExt, itoa};
use rustc_hash::FxHashMap;

use super::stringify_dynamic_chunk_map;
use super::stringify_static_chunk_map;
use crate::{get_chunk_runtime_requirements, runtime_module::unquoted_stringify};

type GetChunkFilenameAllChunks = Box<dyn Fn(&RuntimeGlobals) -> bool + Sync + Send>;
type GetFilenameForChunk = Box<dyn Fn(&Chunk, &Compilation) -> Option<Filename> + Sync + Send>;

#[impl_runtime_module]
pub struct GetChunkFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  #[cacheable(with=Unsupported)]
  content_type: &'static str,
  source_type: SourceType,
  global: String,
  #[cacheable(with=Unsupported)]
  all_chunks: GetChunkFilenameAllChunks,
  #[cacheable(with=Unsupported)]
  filename_for_chunk: GetFilenameForChunk,
}

impl fmt::Debug for GetChunkFilenameRuntimeModule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GetChunkFilenameRuntimeModule")
      .field("id", &self.id)
      .field("chunk", &self.chunk)
      .field("content_type", &self.content_type)
      .field("source_type", &self.source_type)
      .field("global", &self.global)
      .field("all_chunks", &"...")
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
    content_type: &'static str,
    name: &'static str,
    source_type: SourceType,
    global: String,
    all_chunks: F,
    filename_for_chunk: T,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!("webpack/runtime/get {name} chunk filename")),
      None,
      content_type,
      source_type,
      global,
      Box::new(all_chunks),
      Box::new(filename_for_chunk),
    )
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_chunk_filename.ejs").to_string(),
    )]
  }

  fn dependent_hash(&self) -> bool {
    true
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunks = self
      .chunk
      .and_then(|chunk_ukey| compilation.chunk_by_ukey.get(&chunk_ukey))
      .map(|chunk| {
        let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());
        if (self.all_chunks)(runtime_requirements) {
          chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
        } else {
          let mut chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
          if ChunkGraph::get_tree_runtime_requirements(compilation, &chunk.ukey())
            .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
          {
            chunks.extend(
              compilation
                .chunk_graph
                .get_chunk_entry_dependent_chunks_iterable(
                  &chunk.ukey(),
                  &compilation.chunk_by_ukey,
                  &compilation.chunk_group_by_ukey,
                ),
            );
          }
          for entrypoint in
            chunk.get_all_referenced_async_entrypoints(&compilation.chunk_group_by_ukey)
          {
            let entrypoint = compilation.chunk_group_by_ukey.expect_get(&entrypoint);
            chunks.insert(entrypoint.get_entrypoint_chunk());
          }
          chunks
        }
      });

    let mut dynamic_filename: Option<String> = None;
    let mut max_chunk_set_size = 0;
    let mut chunk_filenames = Vec::<(Filename, ChunkUkey)>::new();
    let mut chunk_set_sizes_by_filenames = FxHashMap::<String, usize>::default();
    let mut chunk_map = UkeyIndexMap::default();

    if let Some(chunks) = chunks {
      chunks
        .iter()
        .filter_map(|chunk_ukey| compilation.chunk_by_ukey.get(chunk_ukey))
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

    let dynamic_url = dynamic_filename.as_ref().map(|dynamic_filename| {
      let chunks = chunk_filenames
        .iter()
        .filter_map(|(filename, chunk)| {
          if filename.template() == Some(dynamic_filename.as_str()) {
            Some(*chunk)
          } else {
            None
          }
        })
        .collect::<UkeyIndexSet<ChunkUkey>>();
      let (fake_filename, hash_len_map) =
        get_filename_without_hash_length(&FilenameTemplate::from(dynamic_filename.to_string()));

      let chunk_id = "\" + chunkId + \"";
      let chunk_name = stringify_dynamic_chunk_map(
        |c| {
          c.name_for_filename_template(&compilation.chunk_ids_artifact)
            .map(|s| s.to_string())
        },
        &chunks,
        &chunk_map,
        compilation,
      );
      let chunk_hash = stringify_dynamic_chunk_map(
        |c| {
          let hash = c
            .rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            )
            .map(|hash| hash.to_string());
          match hash_len_map.get("[chunkhash]") {
            Some(hash_len) => hash.map(|s| s[..*hash_len].to_string()),
            None => hash,
          }
        },
        &chunks,
        &chunk_map,
        compilation,
      );
      let content_hash = stringify_dynamic_chunk_map(
        |c| {
          c.rendered_content_hash_by_source_type(
            &compilation.chunk_hashes_artifact,
            &self.source_type,
            compilation.options.output.hash_digest_length,
          )
          .map(|hash| match hash_len_map.get("[contenthash]") {
            Some(hash_len) => hash[..*hash_len].to_string(),
            None => hash.to_string(),
          })
        },
        &chunks,
        &chunk_map,
        compilation,
      );
      let full_hash = match hash_len_map
        .get("[fullhash]")
        .or(hash_len_map.get("[hash]"))
      {
        Some(hash_len) => format!(
          "\" + {}().slice(0, {}) + \"",
          RuntimeGlobals::GET_FULL_HASH,
          itoa!(*hash_len)
        ),
        None => format!("\" + {}() + \"", RuntimeGlobals::GET_FULL_HASH),
      };

      compilation
        .get_path(
          &Filename::<NoFilenameFn>::from(
            serde_json::to_string(fake_filename.as_str()).expect("invalid json to_string"),
          ),
          PathData::default()
            .chunk_id(chunk_id)
            .chunk_hash(&chunk_hash)
            .chunk_name(&chunk_name)
            .hash(&full_hash)
            .content_hash(&content_hash),
        )
        .always_ok()
    });

    let mut static_urls = IndexMap::new();
    for (filename_template, chunk_ukey) in
      chunk_filenames
        .iter()
        .filter(|(filename, _)| match &dynamic_filename {
          None => true,
          Some(dynamic_filename) => filename.template() != Some(dynamic_filename.as_str()),
        })
    {
      if let Some(chunk) = chunk_map.get(chunk_ukey) {
        let (fake_filename, hash_len_map) = get_filename_without_hash_length(filename_template);

        let chunk_id = chunk
          .id(&compilation.chunk_ids_artifact)
          .map(|chunk_id| unquoted_stringify(Some(chunk_id), chunk_id.as_str()));
        let chunk_name = match chunk.name() {
          Some(chunk_name) => Some(unquoted_stringify(
            chunk.id(&compilation.chunk_ids_artifact),
            chunk_name,
          )),
          None => chunk
            .id(&compilation.chunk_ids_artifact)
            .map(|chunk_id| unquoted_stringify(Some(chunk_id), chunk_id.as_str())),
        };
        let chunk_hash = chunk
          .rendered_hash(
            &compilation.chunk_hashes_artifact,
            compilation.options.output.hash_digest_length,
          )
          .map(|chunk_hash| {
            let hash = unquoted_stringify(chunk.id(&compilation.chunk_ids_artifact), chunk_hash);
            match hash_len_map.get("[chunkhash]") {
              Some(hash_len) => hash[..*hash_len].to_string(),
              None => hash,
            }
          });
        let content_hash = chunk
          .content_hash(&compilation.chunk_hashes_artifact)
          .and_then(|content_hash| content_hash.get(&self.source_type))
          .map(|i| {
            let hash = unquoted_stringify(
              chunk.id(&compilation.chunk_ids_artifact),
              i.rendered(compilation.options.output.hash_digest_length),
            );
            match hash_len_map.get("[contenthash]") {
              Some(hash_len) => hash[..*hash_len].to_string(),
              None => hash,
            }
          });
        let full_hash = match hash_len_map
          .get("[fullhash]")
          .or(hash_len_map.get("[hash]"))
        {
          Some(hash_len) => format!(
            "\" + {}().slice(0, {}) + \"",
            RuntimeGlobals::GET_FULL_HASH,
            itoa!(*hash_len)
          ),
          None => format!("\" + {}() + \"", RuntimeGlobals::GET_FULL_HASH),
        };

        let filename = compilation
          .get_path(
            &Filename::<NoFilenameFn>::from(
              serde_json::to_string(
                fake_filename
                  .render(
                    PathData::default()
                      .chunk_name_optional(chunk.name())
                      .chunk_id_optional(
                        chunk
                          .id(&compilation.chunk_ids_artifact)
                          .map(|id| id.as_str()),
                      ),
                    None,
                  )?
                  .as_str(),
              )
              .expect("invalid json to_string"),
            ),
            PathData::default()
              .chunk_id_optional(chunk_id.as_deref())
              .chunk_hash_optional(chunk_hash.as_deref())
              .chunk_name_optional(chunk_name.as_deref())
              .hash(&full_hash)
              .content_hash_optional(content_hash.as_deref()),
          )
          .always_ok();

        if let Some(chunk_id) = chunk.id(&compilation.chunk_ids_artifact) {
          static_urls
            .entry(filename)
            .or_insert(Vec::new())
            .push(chunk_id.as_str());
        }
      }
    }

    let source = compilation.runtime_template.render(&self.id, Some(serde_json::json!({
      "_global": self.global,
      "_static_urls": static_urls
                        .iter()
                        .map(|(filename, chunk_ids)| stringify_static_chunk_map(filename, chunk_ids))
                        .join("\n"),
      "_dynamic_url": dynamic_url.unwrap_or_else(|| format!("\"\" + chunkId + \".{}\"", self.content_type))
    })))?;

    Ok(RawStringSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
