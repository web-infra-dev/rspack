use std::fmt::Write as _;

use rspack_core::{
  Chunk, ChunkLoading, ChunkUkey, Compilation, PathData, RuntimeCodeTemplate, SourceType,
  chunk_graph_chunk::{ChunkId, ChunkIdSet},
  get_js_chunk_filename_template, get_undo_path,
};
use rspack_error::Result;
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  test::is_hot_test,
};

pub fn get_initial_chunk_ids(
  chunk: Option<ChunkUkey>,
  compilation: &Compilation,
  filter_fn: impl Fn(&ChunkUkey, &Compilation) -> bool,
) -> ChunkIdSet {
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
          .collect::<ChunkIdSet>();
        js_chunks.insert(chunk.expect_id().clone());
        js_chunks
      }
      None => ChunkIdSet::default(),
    },
    None => ChunkIdSet::default(),
  }
}

pub fn stringify_chunks(chunks: &ChunkIdSet, value: u8) -> String {
  let mut v = chunks.iter().collect::<Vec<_>>();
  v.sort_unstable();

  let mut result = String::with_capacity(v.len() * 8 + 2);
  result.push('{');
  for chunk_id in v {
    let key = rspack_util::json_stringify(chunk_id);
    result.reserve(key.len() + 4);
    result.push_str(&key);
    result.push_str(": ");
    write!(result, "{value},").expect("infallible write to String");
  }
  result.push('}');
  result
}

pub fn chunk_has_css(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .has_chunk_module_by_source_type(chunk, SourceType::Css, compilation.get_module_graph())
}

pub async fn get_output_dir(
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

pub fn unquoted_stringify(chunk_id: Option<&ChunkId>, str: &str) -> String {
  if let Some(chunk_id) = chunk_id
    && str.len() >= 5
    && str == chunk_id.as_str()
  {
    return "\" + chunkId + \"".to_string();
  }
  let result = rspack_util::json_stringify_str(str);
  result[1..result.len() - 1].to_string()
}

pub fn stringify_dynamic_chunk_map<F>(
  f: F,
  chunks: &FxIndexSet<ChunkUkey>,
  chunk_map: &FxIndexMap<ChunkUkey, &Chunk>,
) -> String
where
  F: Fn(&Chunk) -> Option<String>,
{
  let mut entries = Vec::with_capacity(chunks.len());
  let mut use_id = false;

  for chunk_ukey in chunks.iter() {
    if let Some(chunk) = chunk_map.get(chunk_ukey)
      && let Some(chunk_id) = chunk.id()
      && let Some(value) = f(chunk)
    {
      if value.as_str() == chunk_id.as_str() {
        use_id = true;
      } else {
        entries.push((chunk_id, rspack_util::json_stringify_str(&value)));
      }
    }
  }

  let content = match entries.as_mut_slice() {
    [] => "chunkId".to_string(),
    [(chunk_id, value)] => {
      if use_id {
        format!(
          "(chunkId === {} ? {} : chunkId)",
          rspack_util::json_stringify(*chunk_id),
          value
        )
      } else {
        value.clone()
      }
    }
    entries => {
      let map = stringify_map(entries);
      if use_id {
        format!("({map}[chunkId] || chunkId)")
      } else {
        format!("{map}[chunkId]")
      }
    }
  };
  format!("\" + {content} + \"")
}

pub fn stringify_static_chunk_map(filename: &str, chunk_ids: &[&ChunkId]) -> String {
  let condition = if chunk_ids.len() == 1 {
    let mut condition = String::from("chunkId === ");
    let chunk_id = chunk_ids.first().expect("should have one chunk id");
    condition.push_str(&rspack_util::json_stringify(*chunk_id));
    condition
  } else {
    let mut sorted_chunk_ids = chunk_ids.to_vec();
    sorted_chunk_ids.sort_unstable();

    let mut condition = String::with_capacity(sorted_chunk_ids.len() * 8 + 14);
    condition.push('{');
    condition.push(' ');
    for (idx, chunk_id) in sorted_chunk_ids.iter().enumerate() {
      if idx != 0 {
        condition.push(',');
      }
      let key = rspack_util::json_stringify(*chunk_id);
      condition.reserve(key.len() + 2);
      condition.push_str(&key);
      condition.push_str(":1");
    }
    condition.push_str(" }[chunkId]");
    condition
  };
  let mut result = String::with_capacity(condition.len() + filename.len() + 14);
  result.push_str("if (");
  result.push_str(&condition);
  result.push_str(") return ");
  result.push_str(filename);
  result.push(';');
  result
}

fn stringify_map<T: std::fmt::Display>(entries: &mut [(&ChunkId, T)]) -> String {
  entries.sort_unstable_by_key(|(left, _)| *left);

  let mut result = String::with_capacity(entries.len() * 8 + 2);
  result.push('{');
  for (chunk_id, value) in entries.iter() {
    let key = rspack_util::json_stringify(*chunk_id);
    result.reserve(key.len() + 4);
    result.push_str(&key);
    result.push_str(": ");
    write!(result, "{value},").expect("infallible write to String");
  }
  result.push('}');
  result
}

pub fn generate_javascript_hmr_runtime(
  key: &str,
  method: &str,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<String> {
  runtime_template.render(
    key,
    Some(serde_json::json!({
      "_loading_method": method,
      "_is_hot_test": is_hot_test(),
    })),
  )
}

#[cfg(test)]
mod tests {
  use rspack_core::chunk_graph_chunk::{ChunkId, ChunkIdSet};

  use super::{stringify_chunks, stringify_map, stringify_static_chunk_map};

  #[test]
  fn stringify_chunks_keeps_sorted_numeric_ids() {
    let mut chunks = ChunkIdSet::default();
    chunks.insert(ChunkId::from("2"));
    chunks.insert(ChunkId::from("10"));

    assert_eq!(stringify_chunks(&chunks, 1), "{10: 1,2: 1,}");
  }

  #[test]
  fn stringify_map_keeps_sorted_and_quoted_values() {
    let chunk_a = ChunkId::from("a");
    let chunk_b = ChunkId::from("b");
    let mut entries = vec![
      (&chunk_b, rspack_util::json_stringify_str("beta")),
      (&chunk_a, rspack_util::json_stringify_str("alpha")),
    ];

    assert_eq!(
      stringify_map(&mut entries),
      r#"{"a": "alpha","b": "beta",}"#
    );
  }

  #[test]
  fn stringify_static_chunk_map_single_chunk_keeps_condition_shape() {
    let filename = "\"style.css\"".to_string();
    let chunk_id = ChunkId::from("main");

    assert_eq!(
      stringify_static_chunk_map(&filename, &[&chunk_id]),
      r#"if (chunkId === "main") return "style.css";"#
    );
  }

  #[test]
  fn stringify_static_chunk_map_multiple_chunks_keeps_sorted_object_shape() {
    let filename = "\"style.css\"".to_string();
    let chunk_a = ChunkId::from("b");
    let chunk_b = ChunkId::from("a");

    assert_eq!(
      stringify_static_chunk_map(&filename, &[&chunk_a, &chunk_b]),
      r#"if ({ "a":1,"b":1 }[chunkId]) return "style.css";"#
    );
  }
}
