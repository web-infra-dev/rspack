use crate::{hmr::hmr_module, js_module::JsModule, syntax, BundleOptions};
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use std::{
  collections::HashMap,
  path::Path,
  sync::{Arc, Mutex},
};
use swc::{
  config::{Config, JscConfig, Options, TransformConfig},
  Compiler,
};
use swc_common::{FileName, Mark};
use swc_ecma_transforms_base::pass::noop;
#[derive(Debug, Default)]
pub struct Chunk {
  pub id: String,
  // pub order_modules: Vec<String>,
  pub entry: String,
  pub module_ids: Vec<String>,
  pub source_chunks: Vec<NodeIndex>,
}

impl Chunk {
  pub fn new(module_ids: Vec<String>, entries: String) -> Self {
    Self {
      id: Default::default(),
      module_ids,
      entry: entries,
      source_chunks: Default::default(),
    }
  }

  pub fn from_js_module(module_id: String) -> Self {
    Self {
      id: Default::default(),
      module_ids: vec![module_id.clone()],
      entry: module_id,
      source_chunks: Default::default(),
    }
  }

  pub fn render(
    &mut self,
    _options: &BundleOptions,
    modules: &mut HashMap<String, JsModule>,
    compiler: Arc<Compiler>,
  ) -> RenderedChunk {
    // let compiler = get_compiler();
    let top_level_mark = Mark::from_u32(1);

    let mut output_code = String::new();
    self.module_ids.sort_by_key(|id| modules[id].exec_order);
    self
      .module_ids
      .par_iter()
      .map(|idx| {
        let module = modules.get(idx).unwrap();
        swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
          module.render(&compiler, handler, top_level_mark, modules)
        })
        .unwrap()
      })
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|transform_output| output_code += &transform_output.code);

    RenderedChunk {
      code: output_code,
      file_name: self.id.clone().into(),
    }
  }

  pub fn get_chunk_info_with_file_names(&self) -> OutputChunk {
    OutputChunk {
      code: "".to_string(),
      file_name: self.id.clone().into(),
    }
  }

  #[inline]
  pub fn get_fallback_chunk_name(&self) -> &str {
    get_alias_name(&self.entry)
  }

  #[inline]
  pub fn name(&self) -> &str {
    self.get_fallback_chunk_name()
  }

  pub fn generate_id(&self, options: &BundleOptions) -> String {
    let pattern = &options.entry_file_names;
    pattern.replace("[name]", self.name()).into()
  }
}

#[inline]
fn get_alias_name(id: &str) -> &str {
  let p = Path::new(id);
  // +1 to include `.`
  let ext_len = p.extension().map_or(0, |s| s.to_string_lossy().len() + 1);
  let file_name = p.file_name().unwrap().to_str().unwrap();
  &file_name[0..file_name.len() - ext_len]
}

#[derive(Debug)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
}

#[derive(Debug)]
pub struct RenderedChunk {
  pub code: String,
  pub file_name: String,
}
