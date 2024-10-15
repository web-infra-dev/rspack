use std::{cmp::Ordering, fmt};

use indexmap::IndexMap;
use itertools::Itertools;
use rspack_cacheable::cacheable;
use rspack_collections::{Identifier, UkeyIndexMap, UkeyIndexSet};
use rspack_core::{
  get_chunk_from_ukey, get_css_chunk_filename_template, get_filename_without_hash_length,
  get_js_chunk_filename_template, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Chunk, ChunkUkey, Compilation, Filename, FilenameTemplate, PathData, RuntimeGlobals,
  RuntimeModule, SourceType,
};
use rspack_util::{infallible::ResultInfallibleExt, itoa};
use rustc_hash::FxHashMap;

use super::create_fake_chunk;
use super::stringify_dynamic_chunk_map;
use super::stringify_static_chunk_map;
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::{chunk_has_css, unquoted_stringify},
};

#[cacheable]
pub enum GetChunkFilenameType {
  Css,
  MiniCss(Filename, Filename),
  Javascript,
}

impl GetChunkFilenameType {
  fn all_chunks(&self, runtime_requirements: &RuntimeGlobals) -> bool {
    match self {
      GetChunkFilenameType::Css => {
        runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
      }
      GetChunkFilenameType::MiniCss(_, _) => false,
      GetChunkFilenameType::Javascript => false,
    }
  }
  fn filename_for_chunk(
    &self,
    source_type: &SourceType,
    chunk: &Chunk,
    compilation: &Compilation,
  ) -> Option<Filename> {
    match self {
      GetChunkFilenameType::Css => chunk_has_css(&chunk.ukey, compilation).then(|| {
        get_css_chunk_filename_template(
          chunk,
          &compilation.options.output,
          &compilation.chunk_group_by_ukey,
        )
        .clone()
      }),
      GetChunkFilenameType::MiniCss(filename, chunk_filename) => {
        chunk.content_hash.contains_key(source_type).then(|| {
          if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
            filename.clone()
          } else {
            chunk_filename.clone()
          }
        })
      }
      GetChunkFilenameType::Javascript => Some(
        get_js_chunk_filename_template(
          chunk,
          &compilation.options.output,
          &compilation.chunk_group_by_ukey,
        )
        .clone(),
      ),
    }
  }
}

#[impl_runtime_module]
pub struct GetChunkFilenameRuntimeModule {
  id: Identifier,
  filename_type: GetChunkFilenameType,
  chunk: Option<ChunkUkey>,
  source_type: SourceType,
  global: String,
}

impl fmt::Debug for GetChunkFilenameRuntimeModule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GetChunkFilenameRuntimeModule")
      .field("id", &self.id)
      .field("chunk", &self.chunk)
      .field("source_type", &self.source_type)
      .field("global", &self.global)
      .finish()
  }
}

// It's render is different with webpack, rspack will only render chunk map<chunkId, chunkName>
// and search it.
impl GetChunkFilenameRuntimeModule {
  pub fn new(filename_type: GetChunkFilenameType, source_type: SourceType, global: String) -> Self {
    let name = match &filename_type {
      GetChunkFilenameType::Css => "css",
      GetChunkFilenameType::MiniCss(_, _) => "mini-css",
      GetChunkFilenameType::Javascript => "javascript",
    };
    Self::with_default(
      Identifier::from(format!("webpack/runtime/get {name} chunk filename")),
      filename_type,
      None,
      source_type,
      global,
    )
  }

  fn content_type(&self) -> &str {
    match self.filename_type {
      GetChunkFilenameType::Css => "css",
      GetChunkFilenameType::MiniCss(_, _) => "css",
      GetChunkFilenameType::Javascript => "javascript",
    }
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn cacheable(&self) -> bool {
    false
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunks = self
      .chunk
      .and_then(|chunk_ukey| get_chunk_from_ukey(&chunk_ukey, &compilation.chunk_by_ukey))
      .map(|chunk| {
        let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);
        if self.filename_type.all_chunks(runtime_requirements) {
          chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
        } else {
          let mut chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
          if compilation
            .chunk_graph
            .get_tree_runtime_requirements(&chunk.ukey)
            .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
          {
            chunks.extend(
              compilation
                .chunk_graph
                .get_chunk_entry_dependent_chunks_iterable(
                  &chunk.ukey,
                  &compilation.chunk_by_ukey,
                  &compilation.chunk_group_by_ukey,
                ),
            );
          }
          for entrypoint in
            chunk.get_all_referenced_async_entrypoints(&compilation.chunk_group_by_ukey)
          {
            let entrypoint = compilation.chunk_group_by_ukey.expect_get(&entrypoint);
            chunks.insert(entrypoint.get_entry_point_chunk());
          }
          chunks
        }
      });

    let mut dynamic_filename: Option<String> = None;
    let mut max_chunk_set_size = 0;
    let mut chunk_filenames = Vec::<(Filename, &ChunkUkey)>::new();
    let mut chunk_set_sizes_by_filenames = FxHashMap::<String, usize>::default();
    let mut chunk_map = UkeyIndexMap::default();

    if let Some(chunks) = chunks {
      chunks
        .iter()
        .filter_map(|chunk_ukey| get_chunk_from_ukey(chunk_ukey, &compilation.chunk_by_ukey))
        .for_each(|chunk| {
          let filename =
            self
              .filename_type
              .filename_for_chunk(&self.source_type, chunk, compilation);

          if let Some(filename) = filename {
            chunk_map.insert(&chunk.ukey, chunk);

            chunk_filenames.push((filename.clone(), &chunk.ukey));

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
        .collect::<UkeyIndexSet<&ChunkUkey>>();
      let (fake_filename, hash_len_map) =
        get_filename_without_hash_length(&FilenameTemplate::from(dynamic_filename.to_string()));
      let fake_chunk = create_fake_chunk(
        Some("\" + chunkId + \"".to_string()),
        Some(stringify_dynamic_chunk_map(
          |c| match &c.name {
            Some(name) => Some(name.to_string()),
            None => c.id.clone().map(|id| id.to_string()),
          },
          &chunks,
          &chunk_map,
        )),
        Some(stringify_dynamic_chunk_map(
          |c| {
            let hash = c.rendered_hash.as_ref().map(|hash| hash.to_string());
            match hash_len_map.get("[chunkhash]") {
              Some(hash_len) => hash.map(|s| s[..*hash_len].to_string()),
              None => hash,
            }
          },
          &chunks,
          &chunk_map,
        )),
      );

      let content_hash = Some(stringify_dynamic_chunk_map(
        |c| {
          c.content_hash.get(&self.source_type).map(|i| {
            let hash = i
              .rendered(compilation.options.output.hash_digest_length)
              .to_string();
            match hash_len_map.get("[contenthash]") {
              Some(hash_len) => hash[..*hash_len].to_string(),
              None => hash,
            }
          })
        },
        &chunks,
        &chunk_map,
      ));

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

      format!(
        "\"{}\"",
        compilation
          .get_path(
            &fake_filename,
            PathData::default()
              .chunk(&fake_chunk)
              .hash_optional(Some(full_hash.as_str()))
              .content_hash_optional(content_hash.as_deref())
          )
          .always_ok()
      )
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

        let fake_chunk = create_fake_chunk(
          chunk
            .id
            .as_ref()
            .map(|chunk_id| unquoted_stringify(chunk, chunk_id)),
          match &chunk.name {
            Some(chunk_name) => Some(unquoted_stringify(chunk, chunk_name)),
            None => chunk
              .id
              .as_ref()
              .map(|chunk_id| unquoted_stringify(chunk, chunk_id)),
          },
          chunk.rendered_hash.as_ref().map(|chunk_hash| {
            let hash = unquoted_stringify(chunk, &chunk_hash.as_ref().to_string());
            match hash_len_map.get("[chunkhash]") {
              Some(hash_len) => hash[..*hash_len].to_string(),
              None => hash,
            }
          }),
        );

        let content_hash = chunk.content_hash.get(&self.source_type).map(|i| {
          let hash = unquoted_stringify(
            chunk,
            &i.rendered(compilation.options.output.hash_digest_length)
              .to_string(),
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

        let filename = format!(
          "\"{}\"",
          compilation.get_path(
            &fake_filename,
            PathData::default()
              .chunk(&fake_chunk)
              .hash_optional(Some(full_hash.as_str()))
              .content_hash_optional(content_hash.as_deref())
              .content_hash_type(self.source_type),
          )?,
        );

        if let Some(chunk_id) = &chunk.id {
          static_urls
            .entry(filename)
            .or_insert(Vec::new())
            .push(chunk_id);
        }
      }
    }

    Ok(
      RawSource::from(format!(
        "// This function allow to reference chunks
        {} = function (chunkId) {{
          // return url for filenames not based on template
          {}
          // return url for filenames based on template
          return {};
        }};
      ",
        self.global,
        static_urls
          .iter()
          .map(|(filename, chunk_ids)| stringify_static_chunk_map(filename, chunk_ids))
          .join("\n"),
        dynamic_url.unwrap_or_else(|| format!("\"\" + chunkId + \".{}\"", self.content_type()))
      ))
      .boxed(),
    )
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
