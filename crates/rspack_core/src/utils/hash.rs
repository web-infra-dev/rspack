use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
// use rspack_error::Result;
use rspack_sources::Source;
use xxhash_rust::xxh3::Xxh3;

// use crate::{Compilation, ModuleGraph, SourceType, Ukey};

pub fn get_contenthash<T: Source + Hash>(source: &T) -> u64 {
  let mut xxh3 = Xxh3::new();
  source.hash(&mut xxh3);
  xxh3.finish()
}

pub fn calc_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

// pub fn get_chunkhash(
//   compilation: &Compilation,
//   chunk_ukey: &Ukey,
//   module_graph: &ModuleGraph,
// ) -> u64 {
//   let all_modules = compilation
//     .chunk_graph
//     .get_chunk_modules(chunk_ukey, module_graph)
//     .par_iter()
//     .map(|module| {
//       if module.module.source_types().contains(&SourceType::Css) {
//         module.module.render(SourceType::Css, module, compilation)
//       } else if module
//         .module
//         .source_types()
//         .contains(&SourceType::JavaScript)
//       {
//         module
//           .module
//           .render(SourceType::JavaScript, module, compilation)
//       } else {
//         Ok(None)
//       }
//     })
//     .collect::<Result<Vec<_>>>();

//   get_modules_hash(all_modules.as_ref().expect("TODO:"))
// }

// pub fn get_hash(compilation: &Compilation) -> u64 {
//   let all_modules = compilation
//     .module_graph
//     .modules()
//     .map(|module| {
//       if module.module.source_types().contains(&SourceType::Css) {
//         module.module.render(SourceType::Css, module, compilation)
//       } else if module
//         .module
//         .source_types()
//         .contains(&SourceType::JavaScript)
//       {
//         module
//           .module
//           .render(SourceType::JavaScript, module, compilation)
//       } else {
//         Ok(None)
//       }
//     })
//     .collect::<Result<Vec<_>>>();

//   get_modules_hash(all_modules.as_ref().expect("TODO:"))
// }

// fn get_modules_hash(sources: &[Option<BoxSource>]) -> u64 {
//   let mut xxh3 = Xxh3::new();

//   for source in sources.iter().flatten() {
//     source.hash(&mut xxh3);
//   }

//   xxh3.finish()
// }
