use crate::{Bundle, NormalizedBundleOptions};
use dashmap::DashMap;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use rspack_sources::{
  ConcatSource, GenMapOption, RawSource, Source, SourceMapSource, SourceMapSourceOptions,
};
use rspack_swc::{
  swc::{self, TransformOutput},
  swc_common,
};
use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};
use swc::Compiler;
use swc_common::Mark;
use tracing::instrument;
#[derive(Debug, Default)]
pub struct Chunk {
  pub id: String,
  // pub order_modules: Vec<String>,
  pub entry: String,
  pub module_ids: Vec<String>,
  pub source_chunks: Vec<NodeIndex>,
  pub is_entry_chunk: bool,
  _noop: (),
}

impl Chunk {
  pub fn new(module_ids: Vec<String>, entries: String, is_entry_chunk: bool) -> Self {
    Self {
      id: Default::default(),
      module_ids,
      entry: entries,
      source_chunks: Default::default(),
      is_entry_chunk,
      _noop: (),
    }
  }

  pub fn from_js_module(module_id: String, is_entry_chunk: bool) -> Self {
    Self {
      id: Default::default(),
      module_ids: vec![module_id.clone()],
      entry: module_id,
      source_chunks: Default::default(),
      is_entry_chunk,
      _noop: (),
    }
  }

  #[instrument(skip_all)]
  pub fn render(
    &mut self,
    options: &NormalizedBundleOptions,
    compiler: Arc<Compiler>,
    bundle: &Bundle,
    output_modules: &DashMap<String, Arc<TransformOutput>>,
  ) -> RenderedChunk {
    let mut concattables: Vec<Box<dyn Source>> = vec![];
    let modules = &bundle.module_graph.module_by_id;
    self.module_ids.sort_by_key(|id| 0 - modules[id].exec_order);

    let mut not_transformed_module_ids: Vec<String> = vec![];
    for id in self.module_ids.iter() {
      if output_modules.get(id).is_none() {
        not_transformed_module_ids.push(id.clone());
      }
    }

    let rendered_modules = not_transformed_module_ids
      .par_iter()
      .map(|id| {
        let module = modules.get(id).unwrap();
        module.render(&compiler, &modules, options, &bundle.context)
      })
      .collect::<Vec<_>>();

    for (id, output) in not_transformed_module_ids
      .iter()
      .zip(rendered_modules.into_iter())
    {
      output_modules.insert(id.to_string(), Arc::new(output));
    }

    self.module_ids.iter().for_each(|id| {
      let transform_output = output_modules.get(id).unwrap();
      if let Some(map_string) = &transform_output.map.as_ref() {
        let source_map = sourcemap::SourceMap::from_slice(map_string.as_bytes()).unwrap();
        concattables.push(Box::new(SourceMapSource::new(SourceMapSourceOptions {
          source_code: transform_output.code.clone(),
          name: self.id.clone().into(),
          source_map,
          original_source: None,
          inner_source_map: None,
          remove_original_source: false,
        })));
      } else {
        concattables.push(Box::new(RawSource::new(&transform_output.code)));
      }
    });

    let mut concat_source = ConcatSource::new(vec![]);
    concattables.iter_mut().for_each(|concattable| {
      concat_source.add(concattable.as_mut());
    });

    tracing::debug_span!("conncat_modules").in_scope(|| {
      let output_code;
      if let Some(source_map_url) = concat_source
        // FIXME: generate_url is slow now
        .generate_url(&GenMapOption {
          columns: true,
          include_source_contents: true,
          file: self.id.clone().into(),
        })
        .unwrap()
      {
        output_code =
          concat_source.source().to_string() + "\n//# sourceMappingURL=" + &source_map_url;
      } else {
        output_code = concat_source.source().to_string()
      }

      RenderedChunk {
        code: output_code,
        file_name: self.id.clone().into(),
      }
    })
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

  #[instrument()]
  pub fn generate_id(&self, options: &NormalizedBundleOptions, bundle: &Bundle) -> String {
    let pattern = if self.is_entry_chunk {
      &options.entry_filename
    } else {
      &options.chunk_filename
    };
    let name = pattern.replace("[name]", self.name());
    match pattern.contains("contenthash") {
      true => {
        let content_hash = {
          let mut hasher = DefaultHasher::new();
          self.module_ids.iter().for_each(|module_id| {
            let module = &bundle.module_graph.module_by_id[module_id];
            module.ast.hash(&mut hasher);
          });
          hasher.finish()
        };
        name.replace("[contenthash]", &format!("{:x}", content_hash))
      }
      false => name,
    }
  }
}

#[inline]
fn get_alias_name(id: &str) -> &str {
  let p = Path::new(id);
  // +1 to include `.`
  let ext_len = p.extension().map_or(0, |s| s.to_string_lossy().len() + 1);
  let file_name = p.file_name().and_then(|name| name.to_str()).unwrap();
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
