use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use xxhash_rust::xxh3::xxh3_64;

use crate::{Compilation, ModuleGraph, ModuleRenderResult, SourceType, Ukey};
use rspack_error::Result;

pub fn get_contenthash(code: &str) -> u64 {
  xxh3_64(code.as_bytes())
}

pub fn get_chunkhash(
  compilation: &Compilation,
  chunk_ukey: &Ukey,
  module_graph: &ModuleGraph,
) -> u64 {
  let all_modules = compilation
    .chunk_graph
    .get_chunk_modules(chunk_ukey, module_graph)
    .par_iter()
    .map(|module| {
      if module.module.source_types().contains(&SourceType::Css) {
        module.module.render(SourceType::Css, module, compilation)
      } else if module
        .module
        .source_types()
        .contains(&SourceType::JavaScript)
      {
        module
          .module
          .render(SourceType::JavaScript, module, compilation)
      } else {
        Ok(None)
      }
    })
    .collect::<Result<Vec<_>>>();

  get_modules_hash(all_modules.as_ref().unwrap())
}

pub fn get_hash(compilation: &Compilation) -> u64 {
  let all_modules = compilation
    .module_graph
    .modules()
    .map(|module| {
      if module.module.source_types().contains(&SourceType::Css) {
        module.module.render(SourceType::Css, module, compilation)
      } else if module
        .module
        .source_types()
        .contains(&SourceType::JavaScript)
      {
        module
          .module
          .render(SourceType::JavaScript, module, compilation)
      } else {
        Ok(None)
      }
    })
    .collect::<Result<Vec<_>>>();

  get_modules_hash(all_modules.as_ref().unwrap())
}

fn get_modules_hash(modules: &Vec<Option<ModuleRenderResult>>) -> u64 {
  let mut output = String::new();

  for result in modules {
    if let Some(ModuleRenderResult::JavaScript(source)) = result {
      output += "\n\n";
      output += source;
    }

    if let Some(ModuleRenderResult::Css(source)) = result {
      output += "\n\n";
      output += source;
    }
  }

  xxh3_64(output.as_bytes())
}
