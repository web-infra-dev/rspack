use itertools::Itertools;
use rspack_collections::{UkeyIndexMap, UkeyIndexSet};
use rspack_core::{
  Chunk, ChunkLoading, ChunkUkey, Compilation, PathData, RuntimeTemplate, SourceType,
  chunk_graph_chunk::ChunkId, get_js_chunk_filename_template, get_undo_path,
};
use rspack_error::Result;
use rspack_util::test::is_hot_test;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub(crate) fn get_initial_chunk_ids(
  chunk: Option<ChunkUkey>,
  compilation: &Compilation,
  filter_fn: impl Fn(&ChunkUkey, &Compilation) -> bool,
) -> HashSet<ChunkId> {
  match chunk {
    Some(chunk_ukey) => match compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(&chunk_ukey)
    {
      Some(chunk) => {
        let mut js_chunks = chunk
          .get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
          .iter()
          .filter(|key| !(chunk_ukey.eq(key) || filter_fn(key, compilation)))
          .map(|chunk_ukey| {
            let chunk = compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .expect_get(chunk_ukey);
            chunk.expect_id().clone()
          })
          .collect::<HashSet<_>>();
        js_chunks.insert(chunk.expect_id().clone());
        js_chunks
      }
      None => HashSet::default(),
    },
    None => HashSet::default(),
  }
}

pub fn stringify_chunks(chunks: &HashSet<ChunkId>, value: u8) -> String {
  let mut v = chunks.iter().collect::<Vec<_>>();
  v.sort_unstable();

  format!(
    r#"{{{}}}"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev
        + format!(
          r#"{}: {value},"#,
          serde_json::to_string(cur).expect("chunk to_string failed")
        )
        .as_str()
    })
  )
}

pub fn chunk_has_css(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .has_chunk_module_by_source_type(chunk, SourceType::Css, compilation.get_module_graph())
}

pub(crate) async fn get_output_dir(
  chunk: &Chunk,
  compilation: &Compilation,
  enforce_relative: bool,
) -> rspack_error::Result<String> {
  let filename = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
  );
  let output_dir = compilation
    .get_path(
      &filename,
      PathData::default()
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        )),
    )
    .await?;
  Ok(get_undo_path(
    output_dir.as_str(),
    compilation.options.output.path.as_str().to_string(),
    enforce_relative,
  ))
}

pub fn is_enabled_for_chunk(
  chunk_ukey: &ChunkUkey,
  expected: &ChunkLoading,
  compilation: &Compilation,
) -> bool {
  let chunk_loading = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
    .and_then(|chunk| {
      chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    })
    .and_then(|options| options.chunk_loading.as_ref())
    .unwrap_or(&compilation.options.output.chunk_loading);
  chunk_loading == expected
}

pub(crate) fn unquoted_stringify(chunk_id: Option<&ChunkId>, str: &str) -> String {
  if let Some(chunk_id) = chunk_id
    && str.len() >= 5
    && str == chunk_id.as_str()
  {
    return "\" + chunkId + \"".to_string();
  }
  let result = serde_json::to_string(&str).expect("invalid json");
  result[1..result.len() - 1].to_string()
}

pub(crate) fn stringify_dynamic_chunk_map<F>(
  f: F,
  chunks: &UkeyIndexSet<ChunkUkey>,
  chunk_map: &UkeyIndexMap<ChunkUkey, &Chunk>,
) -> String
where
  F: Fn(&Chunk) -> Option<String>,
{
  let mut result = HashMap::default();
  let mut use_id = false;
  let mut last_key = None;
  let mut entries = 0;

  for chunk_ukey in chunks.iter() {
    if let Some(chunk) = chunk_map.get(chunk_ukey)
      && let Some(chunk_id) = chunk.id()
      && let Some(value) = f(chunk)
    {
      if value.as_str() == chunk_id.as_str() {
        use_id = true;
      } else {
        result.insert(
          chunk_id.as_str(),
          serde_json::to_string(&value).expect("invalid json"),
        );
        last_key = Some(chunk_id.as_str());
        entries += 1;
      }
    }
  }

  let content = if entries == 0 {
    "chunkId".to_string()
  } else if entries == 1 {
    if let Some(last_key) = last_key {
      if use_id {
        format!(
          "(chunkId === {} ? {} : chunkId)",
          serde_json::to_string(&last_key).expect("invalid json"),
          result.get(last_key).expect("cannot find last key")
        )
      } else {
        result.get(last_key).expect("cannot find last key").clone()
      }
    } else {
      unreachable!();
    }
  } else if use_id {
    format!("({}[chunkId] || chunkId)", stringify_map(&result))
  } else {
    format!("{}[chunkId]", stringify_map(&result))
  };
  format!("\" + {content} + \"")
}

pub(crate) fn stringify_static_chunk_map(filename: &String, chunk_ids: &[&str]) -> String {
  let condition = if chunk_ids.len() == 1 {
    format!(
      "chunkId === {}",
      serde_json::to_string(&chunk_ids.first()).expect("invalid json to_string")
    )
  } else {
    let content = chunk_ids
      .iter()
      .sorted_unstable()
      .map(|chunk_id| {
        format!(
          "{}:1",
          serde_json::to_string(chunk_id).expect("invalid json to_string")
        )
      })
      .join(",");
    format!("{{ {content} }}[chunkId]")
  };
  format!("if ({condition}) return {filename};")
}

fn stringify_map<T: std::fmt::Display>(map: &HashMap<&str, T>) -> String {
  format!(
    r#"{{{}}}"#,
    map
      .keys()
      .sorted_unstable()
      .fold(String::new(), |prev, cur| {
        prev
          + format!(
            r#"{}: {},"#,
            serde_json::to_string(cur).expect("json stringify failed"),
            map.get(cur).expect("get key from map")
          )
          .as_str()
      })
  )
}

pub(crate) fn generate_javascript_hmr_runtime(
  key: &str,
  method: &str,
  runtime_template: &RuntimeTemplate,
) -> Result<String> {
  runtime_template.render(
    key,
    Some(serde_json::json!({
      "_loading_method": method,
      "_is_hot_test": is_hot_test(),
    })),
  )
}
