use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rspack_core::{
  get_chunk_from_ukey, get_js_chunk_filename_template, stringify_map, Chunk, ChunkKind,
  ChunkLoading, ChunkUkey, Compilation, PathData, SourceType,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub fn get_initial_chunk_ids(
  chunk: Option<ChunkUkey>,
  compilation: &Compilation,
  filter_fn: impl Fn(&ChunkUkey, &Compilation) -> bool,
) -> HashSet<String> {
  match chunk {
    Some(chunk_ukey) => match get_chunk_from_ukey(&chunk_ukey, &compilation.chunk_by_ukey) {
      Some(chunk) => {
        let mut js_chunks = chunk
          .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
          .iter()
          .filter(|key| !(chunk_ukey.eq(key) || filter_fn(key, compilation)))
          .map(|chunk_ukey| {
            let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
            chunk.expect_id().to_string()
          })
          .collect::<HashSet<_>>();
        js_chunks.insert(chunk.expect_id().to_string());
        js_chunks
      }
      None => HashSet::default(),
    },
    None => HashSet::default(),
  }
}

pub fn stringify_chunks(chunks: &HashSet<String>, value: u8) -> String {
  let mut v = Vec::from_iter(chunks.iter());
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

pub fn chunk_has_js(chunk_ukey: &ChunkUkey, compilation: &Compilation) -> bool {
  if compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0
  {
    return true;
  }

  !compilation
    .chunk_graph
    .get_chunk_modules_by_source_type(
      chunk_ukey,
      SourceType::JavaScript,
      compilation.get_module_graph(),
    )
    .is_empty()
}

pub fn chunk_has_css(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
  !compilation
    .chunk_graph
    .get_chunk_modules_by_source_type(chunk, SourceType::Css, compilation.get_module_graph())
    .is_empty()
}

pub fn get_undo_path(filename: &str, p: String, enforce_relative: bool) -> String {
  let mut depth: i32 = -1;
  let mut append = String::new();
  let mut p = p;
  if p.ends_with('/') || p.ends_with('\\') {
    p.pop();
  }
  for part in filename.split(&['/', '\\']) {
    if part == ".." {
      if depth > -1 {
        depth -= 1
      } else {
        let pos = match (p.rfind('/'), p.rfind('\\')) {
          (None, None) => {
            p.push('/');
            return p;
          }
          (None, Some(j)) => j,
          (Some(i), None) => i,
          (Some(i), Some(j)) => usize::max(i, j),
        };
        append = format!("{}/{append}", &p[pos + 1..]);
        p = p[0..pos].to_string();
      }
    } else if part != "." {
      depth += 1;
    }
  }

  if depth > 0 {
    format!("{}{append}", "../".repeat(depth as usize))
  } else if enforce_relative {
    format!("./{append}")
  } else {
    append
  }
}

pub fn get_output_dir(chunk: &Chunk, compilation: &Compilation, enforce_relative: bool) -> String {
  let filename = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let output_dir = compilation.get_path(
    filename,
    PathData::default().chunk(chunk).content_hash_optional(
      chunk
        .content_hash
        .get(&SourceType::JavaScript)
        .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
    ),
  );
  get_undo_path(
    output_dir.as_str(),
    compilation.options.output.path.display().to_string(),
    enforce_relative,
  )
}

pub fn is_enabled_for_chunk(
  chunk_ukey: &ChunkUkey,
  expected: &ChunkLoading,
  compilation: &Compilation,
) -> bool {
  let chunk_loading = get_chunk_from_ukey(chunk_ukey, &compilation.chunk_by_ukey)
    .and_then(|chunk| chunk.get_entry_options(&compilation.chunk_group_by_ukey))
    .and_then(|options| options.chunk_loading.as_ref())
    .unwrap_or(&compilation.options.output.chunk_loading);
  chunk_loading == expected
}

pub fn unquoted_stringify(chunk: &Chunk, str: &String) -> String {
  if let Some(chunk_id) = &chunk.id {
    if str.len() >= 5 && str == chunk_id {
      return "\" + chunkId + \"".to_string();
    }
  }
  let result = serde_json::to_string(&str).expect("invalid json to_string");
  result[1..result.len() - 1].to_string()
}

pub fn stringify_dynamic_chunk_map<F>(
  f: F,
  chunks: &IndexSet<&ChunkUkey>,
  chunk_map: &IndexMap<&ChunkUkey, &Chunk>,
) -> String
where
  F: Fn(&Chunk) -> Option<String>,
{
  let mut result = HashMap::default();
  let mut use_id = false;
  let mut last_key = None;
  let mut entries = 0;

  for chunk_ukey in chunks.iter() {
    if let Some(chunk) = chunk_map.get(chunk_ukey) {
      if let Some(chunk_id) = &chunk.id {
        if let Some(value) = f(chunk) {
          if value == *chunk_id {
            use_id = true;
          } else {
            result.insert(
              chunk_id.clone(),
              serde_json::to_string(&value).expect("invalid json to_string"),
            );
            last_key = Some(chunk_id.clone());
            entries += 1;
          }
        }
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
          serde_json::to_string(&last_key).expect("invalid json to_string"),
          result.get(&last_key).expect("cannot find last key value")
        )
      } else {
        result
          .get(&last_key)
          .expect("cannot find last key value")
          .clone()
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

pub fn stringify_static_chunk_map(filename: &String, chunk_ids: &[&String]) -> String {
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
    format!("{{ {} }}[chunkId]", content)
  };
  format!("if ({}) return {};", condition, filename)
}

pub fn create_fake_chunk(
  id: Option<String>,
  name: Option<String>,
  rendered_hash: Option<String>,
) -> Chunk {
  let mut fake_chunk = Chunk::new(None, ChunkKind::Normal);
  fake_chunk.name = name;
  fake_chunk.rendered_hash = rendered_hash.map(|h| h.into());
  fake_chunk.id = id;
  fake_chunk
}

#[test]
fn test_get_undo_path() {
  assert_eq!(get_undo_path("a", "/a/b/c".to_string(), true), "./");
  assert_eq!(
    get_undo_path("static/js/a.js", "/a/b/c".to_string(), false),
    "../../"
  );
}
