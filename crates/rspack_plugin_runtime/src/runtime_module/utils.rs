use hashbrown::HashSet;
use rspack_core::{ChunkUkey, Compilation};

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
) -> HashSet<String> {
  match chunk {
    Some(chunk_ukey) => match compilation.chunk_by_ukey.get(&chunk_ukey) {
      Some(chunk) => {
        let chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);
        let js_chunks = chunks
          .iter()
          .filter_map(|chunk| compilation.chunk_by_ukey.get(chunk))
          .flat_map(|chunk| {
            chunk
              .files
              .iter()
              .filter(|file| file.ends_with(".js"))
              .collect::<Vec<_>>()
          })
          .cloned()
          .collect::<HashSet<_>>();
        js_chunks
      }
      None => HashSet::default(),
    },
    None => HashSet::default(),
  }
}

pub fn stringify_chunks(chunks: &HashSet<String>, value: u8) -> String {
  let mut v = Vec::from_iter(chunks.iter());
  v.sort();

  format!(
    r#"{{{}}}"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{}": {},"#, cur, value).as_str()
    })
  )
}
