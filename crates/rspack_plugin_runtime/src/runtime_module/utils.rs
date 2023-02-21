use rspack_core::{ChunkUkey, Compilation, SourceType};
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

pub fn stringify_chunks_to_array(chunks: &HashSet<String>) -> String {
  let mut v = Vec::from_iter(chunks.iter());
  v.sort_unstable();

  format!(
    r#"[{}]"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{cur}","#).as_str()
    })
  )
}

pub fn stringify_array(vec: &[String]) -> String {
  format!(
    r#"[{}]"#,
    vec.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{cur}","#).as_str()
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

  compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(
      chunk_ukey,
      SourceType::JavaScript,
      &compilation.module_graph,
    )
    .count()
    > 0
}
