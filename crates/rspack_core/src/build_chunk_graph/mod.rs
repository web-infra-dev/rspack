// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use std::{
  collections::HashMap,
  hash::{Hash, Hasher},
};

use code_splitter::CodeSplitter;
use serde::Serialize;
use tracing::instrument;

use crate::Compilation;

pub(crate) mod code_splitter;
pub(crate) mod incremental;

#[instrument(skip_all)]
pub(crate) fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  // let mut splitter = code_splitter::CodeSplitter::new(compilation);
  let mut splitter = compilation.code_splitting_cache.code_splitter.clone();
  splitter.update_with_compilation(compilation)?;

  dbg!(compilation.chunk_group_by_ukey.len());

  if splitter.chunk_group_infos.is_empty() {
    let inputs = splitter.prepare_input_entrypoints_and_modules(compilation)?;
    splitter.prepare_entries(inputs, compilation)?;
  }

  splitter.split(compilation)?;

  splitter.remove_orphan(compilation)?;

  // make sure all module (weak dependency particularly) has a cgm
  let ids = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect::<Vec<_>>();

  for module_identifier in ids {
    compilation.chunk_graph.add_module(module_identifier)
  }
  // write_vis_data(&splitter, &compilation);

  compilation.code_splitting_cache.code_splitter = splitter;

  // debug
  {
    dbg!(compilation.chunk_by_ukey.len());
    dbg!(compilation.chunk_group_by_ukey.len());

    let mut res = vec![];
    for chunk_group in compilation.chunk_group_by_ukey.values() {
      let mut origins = chunk_group
        .origins()
        .into_iter()
        .map(|orig| orig.module_id.unwrap_or_default().to_string())
        .collect::<Vec<_>>();
      origins.sort();
      let mut modules = compilation
        .chunk_graph
        .get_chunk_module_identifiers(&chunk_group.chunks[0])
        .into_iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>();
      modules.sort();

      origins.push("|||".into());

      origins.extend(modules);

      res.push(origins.join("@@@"))
    }
    res.sort();
    let mut hasher = rustc_hash::FxHasher::default();
    res.hash(&mut hasher);
    let hash = hasher.finish();
    dbg!(hash);

    let filename = format!("chunks-{}", hash);
    std::fs::write(filename, res.join("\n")).unwrap();
  }
  // {
  //   let chunk_graph = &compilation.chunk_graph;
  //   let mut modules: Vec<String> = vec![];
  //   for chunk in compilation.chunk_by_ukey.keys().collect::<Vec<_>>() {
  //     modules.extend(
  //       chunk_graph
  //         .get_chunk_module_identifiers(chunk)
  //         .into_iter()
  //         .map(|id| id.to_string()),
  //     );
  //   }
  //   modules.sort();
  //   let mut hasher = rustc_hash::FxHasher::default();
  //   modules.hash(&mut hasher);
  //   let hash = hasher.finish();

  //   let path = format!("modules-{}.txt", hash);
  //   std::fs::write(std::path::Path::new(&path), modules.join("\n"));
  //   let mut res = vec![];
  //   for (ukey, chunk_group) in compilation.chunk_group_by_ukey.iter() {
  //     let chunk = chunk_group.chunks[0];
  //     let mut modules = chunk_graph
  //       .get_chunk_module_identifiers(&chunk)
  //       .into_iter()
  //       .collect::<Vec<_>>();
  //     modules.sort();

  //     res.push(format!(
  //       "origins: {}\n{}",
  //       chunk_group
  //         .origins()
  //         .into_iter()
  //         .map(|orig| orig.module_id.unwrap_or_default())
  //         .join(","),
  //       modules.into_iter().map(|id| id.to_string()).join(",")
  //     ));
  //   }

  //   let path = format!("chunks-{}.txt", hash);
  //   res.sort();
  //   std::fs::write(std::path::Path::new(&path), res.join("\n"));

  //   dbg!(compilation.chunk_group_by_ukey.len());
  // }

  Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsVisData {
  name: Option<String>,
  origins: Vec<String>,
  modules: Vec<String>,
  skipped_modules: Vec<String>,
  children: Vec<u32>,
  available_modules: Vec<String>,
}

#[derive(Serialize)]
struct Data {
  root: Vec<u32>,
  map: HashMap<u32, JsVisData>,
}

fn write_vis_data(splitter: &CodeSplitter, compilation: &Compilation) {
  let mut map = HashMap::default();
  let module_graph = compilation.get_module_graph();
  for (ukey, chunk_group) in compilation.chunk_group_by_ukey.iter() {
    let mut modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(&chunk_group.chunks[0])
      .into_iter()
      .map(|id| {
        let m = module_graph.module_by_identifier(id).unwrap();
        m.readable_identifier(&compilation.options.context)
          .to_string()
      })
      .collect::<Vec<_>>();

    modules.sort();

    let Some(cgi) = splitter
      .chunk_group_info_map
      .get(ukey)
      .and_then(|ukey| splitter.chunk_group_infos.get(ukey))
    else {
      continue;
    };

    let mut skipped_modules = cgi
      .skipped_items
      .iter()
      .map(|id| {
        let m = module_graph.module_by_identifier(id).unwrap();
        m.readable_identifier(&compilation.options.context)
          .to_string()
      })
      .collect::<Vec<_>>();

    skipped_modules.sort();

    let data = JsVisData {
      name: chunk_group.name().map(|s| s.to_string()),
      origins: chunk_group
        .origins()
        .into_iter()
        .filter_map(|rec| rec.module_id.map(|id| id.to_string()))
        .collect(),
      modules,
      skipped_modules,
      children: chunk_group
        .children
        .iter()
        .map(|child| child.as_u32())
        .collect(),
      available_modules: vec![],
    };

    map.insert(ukey.as_u32(), data);
  }

  let root = compilation
    .entrypoints
    .values()
    .into_iter()
    .map(|cgi_ukey| cgi_ukey.as_u32())
    .collect::<Vec<_>>();

  let final_data: Data = Data { root, map };

  let filename = format!("data-{}.json", compilation.id().0);
  std::fs::write(filename, serde_json::to_string(&final_data).unwrap());
}
