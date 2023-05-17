use rspack_core::{
  get_js_chunk_filename_template, Chunk, ChunkUkey, Compilation, PathData, SourceType,
};
use rustc_hash::FxHashSet as HashSet;

// pub fn condition_map_to_string(map: &HashMap<String, bool>, _value: String) -> String {
//   let positive_items = map
//     .iter()
//     .filter(|(_, v)| **v)
//     .map(|(k, _)| k)
//     .collect::<Vec<_>>();
//   if positive_items.len() == 0 {
//     return "false".to_string();
//   }
//   let negative_items = map
//     .iter()
//     .filter(|(_, v)| !**v)
//     .map(|(k, _)| k)
//     .collect::<Vec<_>>();
//   if negative_items.len() == 0 {
//     return "true".to_string();
//   }
//   // TODO
//   return "true".to_string();
// }

pub fn get_initial_chunk_ids(
  chunk: Option<ChunkUkey>,
  compilation: &Compilation,
  filter_fn: impl Fn(&ChunkUkey, &Compilation) -> bool,
) -> HashSet<String> {
  match chunk {
    Some(chunk_ukey) => match compilation.chunk_by_ukey.get(&chunk_ukey) {
      Some(chunk) => {
        let mut js_chunks = chunk
          .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
          .iter()
          .filter(|key| !(chunk_ukey.eq(key) || filter_fn(key, compilation)))
          .map(|chunk_ukey| {
            let chunk = compilation
              .chunk_by_ukey
              .get(chunk_ukey)
              .expect("Chunk not found");
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
      prev + format!(r#""{cur}": {value},"#).as_str()
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
      &compilation.module_graph,
    )
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
        .map(|i| i.as_str()),
    ),
  );
  get_undo_path(
    output_dir.as_str(),
    compilation.options.output.path.display().to_string(),
    enforce_relative,
  )
}

#[test]
fn test_get_undo_path() {
  assert_eq!(get_undo_path("a", "/a/b/c".to_string(), true), "./");
  assert_eq!(
    get_undo_path("static/js/a.js", "/a/b/c".to_string(), false),
    "../../"
  );
}
